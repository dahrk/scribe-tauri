# Product Spec: Meeting Recorder

## Problem

Knowledge from meetings (decisions, action items, context) is lost when
recordings aren't taken or aren't searchable. Existing tools require cloud
uploads, which is unacceptable for confidential business conversations.

## Solution

An always-on background app that:
1. Detects when a meeting is in progress (both mic and system audio active).
2. Transcribes locally using NVIDIA Parakeet v3 – no audio ever leaves the machine.
3. Summarises the transcript with a local LLM (Ollama) into bullet-point action items.
4. Stores only the text in a local SQLite database.
5. Lets the user browse, edit, and search past recordings in a clean sidebar UI.

## User stories

| As a… | I want to… | So that… |
|---|---|---|
| Developer | have the app auto-detect meetings | I don't have to remember to start it |
| Manager | see a bullet summary after a call | I can review action items without reading a full transcript |
| Privacy-conscious user | keep all audio and text on my machine | My conversations aren't uploaded to any cloud |
| Power user | edit the title and transcript | I can annotate and correct ASR errors |
| Power user | regenerate the summary | I can try a different LLM or prompt |

## Out of scope (v1)

- Audio playback / replay
- Export to Markdown / PDF (tracked in [export.md](export.md))
- Speaker diarisation
- Multi-language transcription (Parakeet is English-only)
- Mobile / iOS / Android
