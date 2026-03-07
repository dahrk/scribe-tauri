# Product Sense

## Who is this for?

Knowledge workers who participate in remote meetings daily and lose track of
decisions and action items. Specifically:

- **Privacy-conscious teams** (legal, finance, healthcare) who cannot use
  cloud-based meeting tools.
- **Developers and technical leads** who want local AI tooling they control.
- **Power users** comfortable with running a local Python server for ASR.

## What makes this different?

| Feature | Scribe | Cloud meeting tools (Otter, Fireflies) |
|---|---|---|
| Audio leaves machine | Never | Always |
| Requires internet | No | Yes |
| Cost | Free (hardware) | Subscription |
| Latency | Seconds (local GPU) | Seconds (cloud) |
| Accuracy | High (Parakeet) | High |
| Supports any call app | Yes (captures system audio) | Depends |

## North star metric

**Meetings captured per week per active user.** High capture rate means users
trust the app to run in the background reliably.

## Key UX principles

- **Zero friction start**: auto-record means users do nothing. The app
  handles detection.
- **Transparency**: the status bar always shows current state (Idle /
  Recording + live preview). Users are never surprised.
- **Editability**: transcripts and summaries are editable. ASR makes
  mistakes; users fix them.
- **No lock-in**: all data is in a local SQLite file the user owns.
