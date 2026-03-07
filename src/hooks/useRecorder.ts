import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { RecorderStatus } from "../types";

export function useRecorder() {
  const [status, setStatus] = useState<RecorderStatus>({
    is_recording: false,
    recording_id: null,
    transcript_preview: "",
  });

  useEffect(() => {
    const unlisten = listen<RecorderStatus>("recorder-status", (evt) => {
      setStatus(evt.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const startRecording = useCallback(() => {
    invoke("start_recording").catch(console.error);
  }, []);

  const stopRecording = useCallback(() => {
    invoke("stop_recording").catch(console.error);
  }, []);

  return { status, startRecording, stopRecording };
}
