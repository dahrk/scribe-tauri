#!/usr/bin/env python3
"""
Scribe – Parakeet ASR HTTP server
=====================================
Loads the NVIDIA Parakeet TDT model once and serves transcription requests over
HTTP so the Rust backend can call it without reloading the model on every segment.

Requirements:
    pip install "nemo_toolkit[asr]" fastapi uvicorn

Usage:
    python scripts/parakeet_server.py [--model parakeet-tdt-0.6b-v2] [--port 9000]

The server exposes:
    POST /transcribe
        Content-Type: audio/wav
        Body: raw WAV bytes (16-bit PCM, mono, 16 kHz recommended)
        Response: {"text": "transcribed text"}

    GET /health
        Response: {"status": "ok", "model": "<model name>"}
"""

import argparse
import io
import logging
import os
import sys
import tempfile
from contextlib import asynccontextmanager

import uvicorn
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s – %(message)s")
log = logging.getLogger("parakeet_server")

# ── Globals filled during startup ────────────────────────────────────────────
_asr_model = None
_model_name = None


def load_model(model_name: str):
    """Load Parakeet via NeMo ASR."""
    try:
        import nemo.collections.asr as nemo_asr  # type: ignore
    except ImportError:
        log.error(
            "NeMo not installed. Run: pip install 'nemo_toolkit[asr]'"
        )
        sys.exit(1)

    log.info(f"Loading model '{model_name}' – this may take a moment…")
    # Map short names to Hugging Face / NGC identifiers
    model_map = {
        "parakeet-tdt-0.6b-v2": "nvidia/parakeet-tdt-0.6b-v2",
        "parakeet-tdt-1.1b": "nvidia/parakeet-tdt-1.1b",
        "parakeet-ctc-0.6b": "nvidia/parakeet-ctc-0.6b",
    }
    resolved = model_map.get(model_name, model_name)
    model = nemo_asr.models.ASRModel.from_pretrained(model_name=resolved)
    model.eval()
    log.info(f"Model '{resolved}' ready.")
    return model


@asynccontextmanager
async def lifespan(app: FastAPI):
    global _asr_model
    _asr_model = load_model(_model_name)
    yield
    _asr_model = None


app = FastAPI(title="Scribe Parakeet ASR", lifespan=lifespan)


@app.get("/health")
async def health():
    return {"status": "ok", "model": _model_name}


@app.post("/transcribe")
async def transcribe(request: Request):
    """
    Accept raw WAV bytes and return a JSON object with the transcription.
    The endpoint is called by the Rust backend for every audio segment.
    """
    if _asr_model is None:
        raise HTTPException(status_code=503, detail="Model not loaded")

    body = await request.body()
    if not body:
        raise HTTPException(status_code=400, detail="Empty request body")

    # Write to a temp WAV file (NeMo expects a file path)
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
        f.write(body)
        tmp_path = f.name

    try:
        transcriptions = _asr_model.transcribe([tmp_path])
        text = transcriptions[0] if transcriptions else ""
    except Exception as exc:
        log.exception("Transcription error")
        raise HTTPException(status_code=500, detail=str(exc))
    finally:
        os.unlink(tmp_path)

    return JSONResponse({"text": text.strip()})


def main():
    global _model_name
    parser = argparse.ArgumentParser(description="Scribe Parakeet ASR server")
    parser.add_argument(
        "--model",
        default="parakeet-tdt-0.6b-v2",
        help="Parakeet model name or Hugging Face path (default: parakeet-tdt-0.6b-v2)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=9000,
        help="Port to listen on (default: 9000)",
    )
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help="Host to bind (default: 127.0.0.1)",
    )
    args = parser.parse_args()
    _model_name = args.model

    log.info(f"Starting Scribe Parakeet server on {args.host}:{args.port}")
    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")


if __name__ == "__main__":
    main()
