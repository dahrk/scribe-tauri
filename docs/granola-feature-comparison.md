# Scribe vs Granola – Feature Comparison & Gap Analysis

> Generated: 2026-03-15
> Purpose: identify parity gaps and prioritise implementation todos.
> Granola sources: [granola.ai](https://www.granola.ai), [TechCrunch](https://techcrunch.com/2025/05/14/ai-note-taking-app-granola-raises-43m-at-250m-valuation-launches-collaborative-features/), [Zapier review](https://zapier.com/blog/granola-ai/), [bluedothq review](https://www.bluedothq.com/blog/granola-review), [granola pricing](https://www.granola.ai/pricing)

---

## Status legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Scribe has this today |
| 🚧 | Planned / in-progress in Scribe |
| ❌ | Granola has this; Scribe does not |
| ⬜ | Granola does **not** have this; Scribe does (Scribe advantage) |
| ❓ | Ambiguous – see notes |

---

## 1. Core Recording & Transcription

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Device-level audio capture (no bot joining call) | ✅ | ✅ | — |
| Microphone capture | ✅ | ✅ | — |
| System audio capture | ✅ | ✅ | — |
| Automatic meeting detection (start/stop) | ✅ | ✅ | — |
| Manual start/stop | ✅ | ✅ | — |
| Real-time / live transcript preview | ✅ (status bar) | ✅ | — |
| Multiple ASR backends (Parakeet / Whisper / custom) | ✅ | ❌ (proprietary only) | — (Scribe advantage) |
| Local-only transcription (no cloud ASR) | ✅ | ❌ (cloud transcription) | — (Scribe advantage) |
| Multi-language transcription | 🚧 (planned) | ✅ | [ ] Add multilingual Parakeet variant |
| Speaker diarisation (who said what) | 🚧 (planned) | ✅ | [ ] Add diarisation model integration |
| Phone call recording (mobile outbound) | ❌ | ✅ (iOS app) | ❓ Out of scope? |
| Transcription accuracy toggle / model size | ✅ | ❌ (no user control) | — (Scribe advantage) |

---

## 2. AI Notes & Summarisation

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Automatic meeting summary | ✅ | ✅ | — |
| Multiple LLM backends (Ollama / OpenAI-compat) | ✅ | ❌ (proprietary only) | — (Scribe advantage) |
| Local LLM summarisation (Ollama) | ✅ | ❌ | — (Scribe advantage) |
| Manual summary regeneration | ✅ | ❓ | — |
| Hybrid human + AI notes (user writes, AI fills in) | ❌ | ✅ | [ ] Add in-meeting scratchpad that merges with transcript |
| Meeting templates (sales call, 1:1, standup, etc.) | ❌ | ✅ | [ ] Implement per-meeting-type prompt templates in settings |
| Edit notes by natural language request ("make this shorter") | ❌ | ✅ | [ ] Add AI edit command in RecordingDetail |
| Chat with notes / Q&A over transcript | ❌ | ✅ | [ ] Add chat panel backed by local LLM / OpenAI-compat |
| Cross-meeting Q&A (query weeks of notes at once) | ❌ | ✅ | [ ] Vector-embed transcripts; add semantic search |
| Glance at previous notes mid-meeting | ❌ | ✅ | [ ] Allow navigation to past recordings while recording active |
| Action item / follow-up extraction | ❌ | ✅ (via templates) | [ ] Add action-item extraction to summary prompt |

---

## 3. Storage & Privacy

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Local-only storage (no cloud) | ✅ | ❌ (cloud-synced) | — (Scribe advantage) |
| Audio never persisted to disk | ✅ | ❓ | — |
| SQLite text storage | ✅ | ❌ (proprietary cloud DB) | — |
| User-controlled data directory | ✅ | ❌ | — (Scribe advantage) |
| Organisation-wide AI training opt-out | ❌ | ✅ (enterprise tier) | ❓ N/A if always local |
| SSO / enterprise auth | ❌ | ✅ (enterprise) | ❓ Only needed if multi-user |
| End-to-end encryption at rest | ❌ | ❓ | [ ] Encrypt SQLite at rest (SQLCipher) |

---

## 4. Recording Management & Search

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Chronological recording list | ✅ | ✅ | — |
| Editable title | ✅ | ✅ | — |
| Editable transcript | ✅ | ✅ | — |
| Editable summary | ✅ | ✅ | — |
| Delete recording | ✅ | ✅ | — |
| Keyword search across recordings | ❌ | ✅ | [ ] Add full-text search (SQLite FTS5) |
| Semantic / AI search across recordings | ❌ | ✅ | [ ] Vector embeddings + similarity search |
| Filter by person / attendee | ❌ | ✅ | [ ] Tag recordings with attendees; filter by person |
| Folder / project organisation | ❌ | ✅ | [ ] Add folder/tag concept to DB schema and UI |
| Pagination for large libraries | 🚧 (TD-005) | ✅ | [ ] Implement `list_recordings` pagination |
| Built-in lightweight contact/person tracker | ❌ | ✅ | ❓ Needed? |

---

## 5. Export & Sharing

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Export to Markdown | 🚧 (planned v0.2) | ✅ | [ ] Implement `export_recording(id, Markdown)` |
| Export to PDF | 🚧 (planned v0.2) | ✅ | [ ] Implement `export_recording(id, PDF)` |
| Copy transcript / summary to clipboard | ❌ | ✅ | [ ] Add copy buttons to RecordingDetail |
| Share notes via link (web-accessible) | ❌ | ✅ | ❓ Conflicts with local-only design |
| Share with non-Granola users (chat w/ AI) | ❌ | ✅ | ❓ N/A — no server |
| Export to Notion | ❌ | ✅ | [ ] Add Notion export action (requires user API key) |
| Send to Slack channel | ❌ | ✅ | [ ] Add Slack webhook action (optional) |

---

## 6. Calendar & Meeting Context

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Calendar integration (Google / Outlook) | ❌ | ✅ | [ ] Read local calendar (CalDAV / Google API) to pre-fill meeting title & attendees |
| Auto-attach notes to calendar event | ❌ | ✅ | [ ] Link recordings to calendar event ID |
| Meeting agenda pre-loaded before call | ❌ | ✅ | [ ] Pull agenda from calendar event description |
| Attendee list from calendar event | ❌ | ✅ | [ ] Store attendees in DB; show in RecordingDetail |

---

## 7. Integrations & Automation

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Zapier (8,000+ app automation) | ❌ | ✅ | ❓ Needs cloud webhook endpoint |
| Notion export | ❌ | ✅ | [ ] Notion API export action |
| HubSpot CRM sync | ❌ | ✅ | ❓ Niche; low priority |
| Affinity / Attio CRM sync | ❌ | ✅ | ❓ Niche; low priority |
| Slack post-meeting summary | ❌ | ✅ | [ ] Slack webhook setting (opt-in) |
| MCP (Model Context Protocol) | ❌ | ✅ (Feb 2026) | [ ] Expose local recordings via MCP server |
| Zapier / webhook on recording complete | ❌ | ✅ | [ ] Add configurable webhook URL in settings |
| API access (enterprise) | ❌ | ✅ | ❓ Scope unclear |

---

## 8. Platform Support

| Platform | Scribe | Granola |
|----------|--------|---------|
| macOS | ✅ | ✅ |
| Linux | ✅ | ❌ (Scribe advantage) |
| Windows | ❌ | ✅ |
| iOS | ❌ | ✅ |
| Android | ❌ | ❌ |
| Web app | ❌ | ❌ |
| Lock-screen widget (mobile) | ❌ | ✅ (iOS) |

**Todos:**
- [ ] Windows support (CPAL + ScreenCaptureKit equivalent for Windows WASAPI)
- [ ] iOS / mobile app — ❓ likely out of scope for v1

---

## 9. Collaboration (Team Features)

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Shared team folders | ❌ | ✅ | ❓ Requires server / sync — major architecture shift |
| Per-folder permissions (private vs shared) | ❌ | ✅ | ❓ Same |
| Invite teammates to view notes | ❌ | ✅ | ❓ Same |
| Cross-team AI insights ("What were pain points this quarter?") | ❌ | ✅ | ❓ Same |
| Centralised billing / seat management | ❌ | ✅ | ❓ N/A for local app |

> **Note:** All Granola collaboration features require cloud infrastructure.
> Scribe's local-only design is a deliberate trade-off.
> Decide: should Scribe add an optional sync/server mode?

---

## 10. UX & Workflow Polish

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| System tray presence | ✅ | ✅ | — |
| Global keyboard shortcuts (start/stop) | 🚧 (planned) | ✅ | [ ] Register global hotkey via Tauri plugin |
| Manual audio device selection | 🚧 (TD-003) | ✅ | [ ] Implement device picker in settings |
| Onboarding / setup wizard | ❌ | ✅ | [ ] Add first-run wizard (ASR server setup, permissions) |
| Meeting-type template selector | ❌ | ✅ | [ ] See AI Notes section above |
| Dark / light mode | ❓ | ✅ | [ ] Verify theme support; add if missing |
| Mobile sync (notes appear on all devices) | ❌ | ✅ | ❓ Requires cloud |

---

## 11. Pricing & Business Model

| Aspect | Scribe | Granola |
|--------|--------|---------|
| Model | Open-source / self-hosted | SaaS subscription |
| Free tier | ✅ (fully free, self-hosted) | ✅ (25 meetings, 14-day history) |
| Paid tier | ❌ (no commercial offering) | $14/user/mo (Business) |
| Enterprise | ❌ | $35+/user/mo |
| Data ownership | ✅ Full (local) | ❌ Cloud-hosted |
| Requires internet | ❌ (fully offline) | ✅ |

---

## Open Questions (❓ items to resolve)

1. **Collaboration / sync**: Should Scribe ever offer an optional server-sync mode for teams, or remain strictly local-only?
2. **Phone call recording**: Is iOS / mobile call capture in scope?
3. **Link sharing**: "Share notes via link" requires a server endpoint. Out of scope?
4. **CRM integrations** (HubSpot, Salesforce): Worth building, or leave to webhook/MCP?
5. **Zapier / webhook**: Granola triggers webhooks on meeting-complete. Scribe could do this with a configurable POST URL — is this wanted?
6. **SQLite encryption** (SQLCipher): Granola has org-wide AI training opt-out for enterprise. Scribe's local model avoids this problem, but should the DB be encrypted at rest?
7. **Windows support**: Granola supports Windows; Scribe currently does not. Is this a target platform?
8. **Dark mode**: Does Scribe's current Tauri/React UI respect system theme?
9. **Built-in contact tracker**: Granola has a lightweight "who did I meet with" view. Wanted in Scribe?
10. **MCP server**: Expose Scribe's local recordings via MCP so Claude / ChatGPT can query them. Quick win?

---

## Prioritised todo summary (parallelisable work)

### Group A – Low-hanging fruit (no architecture changes)

- [ ] Copy-to-clipboard buttons for transcript and summary
- [ ] SQLite FTS5 full-text search across recordings
- [ ] Implement recording export: Markdown + PDF (already planned v0.2)
- [ ] Add pagination to `list_recordings` (TD-005)
- [ ] Global keyboard shortcut for start/stop (already planned)
- [ ] Manual audio device picker (TD-003)
- [ ] Action item extraction in summary prompt
- [ ] Dark/light mode verification and fix

### Group B – Medium effort (self-contained features)

- [ ] Per-meeting prompt templates (sales call, 1:1, standup, etc.)
- [ ] AI edit command: "make this shorter / more formal"
- [ ] Notion export action (user provides API key in settings)
- [ ] Slack webhook post-meeting summary (opt-in setting)
- [ ] Configurable generic webhook URL (POST on recording complete)
- [ ] MCP server exposing local recordings (read-only)
- [ ] Calendar integration: read Google/CalDAV to pre-fill title + attendees
- [ ] Folder / tag organisation in DB + UI

### Group C – Larger scope (significant effort)

- [ ] Chat / Q&A panel backed by local LLM (in-meeting and post-meeting)
- [ ] Cross-meeting semantic search (vector embeddings)
- [ ] Speaker diarisation model integration
- [ ] Multi-language transcription (multilingual Parakeet/Whisper)
- [ ] In-meeting scratchpad that merges user notes + transcript (Granola's core UX)
- [ ] First-run onboarding wizard
- [ ] Windows platform support

### Group D – Needs product decision first (❓ items)

- [ ] Optional cloud sync / server mode for team collaboration
- [ ] iOS app
- [ ] Link-sharing (requires server)
- [ ] SQLite encryption at rest (SQLCipher)
- [ ] Built-in contact/person tracker
