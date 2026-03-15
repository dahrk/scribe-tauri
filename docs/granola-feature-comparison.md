# Scribe vs Granola – Feature Comparison & Gap Analysis

> Generated: 2026-03-15
> Purpose: identify parity gaps and prioritise implementation todos.
> Granola sources: [granola.ai](https://www.granola.ai), [TechCrunch $43M raise](https://techcrunch.com/2025/05/14/ai-note-taking-app-granola-raises-43m-at-250m-valuation-launches-collaborative-features/), [Zapier review](https://zapier.com/blog/granola-ai/), [bluedothq review](https://www.bluedothq.com/blog/granola-review), [granola pricing](https://www.granola.ai/pricing), [tldv review](https://tldv.io/blog/granola-review/), [Granola 2.0 blog](https://www.granola.ai/blog/two-dot-zero), [Granola MCP](https://www.granola.ai/blog/granola-mcp)

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
| In-person / room audio (mic only) | ✅ | ✅ | — |
| Automatic meeting detection (start/stop) | ✅ | ✅ (via calendar) | — |
| Manual start/stop | ✅ | ✅ | — |
| Real-time / live transcript preview | ✅ (status bar) | ✅ | — |
| Multiple ASR backends (Parakeet / Whisper / custom) | ✅ | ❌ (proprietary only) | — (Scribe advantage) |
| Local-only transcription (no cloud ASR) | ✅ | ❌ (cloud transcription) | — (Scribe advantage) |
| Multi-language transcription (10+ languages, auto-detect) | 🚧 (planned) | ✅ | [ ] Add multilingual Parakeet variant; detect language |
| Mid-call language switching | ❌ | ✅ | [ ] Depends on multilingual ASR |
| Speaker diarisation (who said what) | 🚧 (planned) | ✅ (manual ID per meeting) | [ ] Add diarisation model; note: Granola requires manual name assignment |
| Phone call recording (mobile outbound) | ❌ | ✅ (iOS app) | ❓ Out of scope? |
| Transcription accuracy toggle / model size | ✅ | ❌ (no user control) | — (Scribe advantage) |
| Audio never stored | ✅ | ✅ | — |

---

## 2. AI Notes & Summarisation

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Automatic meeting summary | ✅ | ✅ | — |
| Multiple LLM backends (Ollama / OpenAI-compat) | ✅ | ❌ (GPT-4o + Claude, proprietary) | — (Scribe advantage) |
| Local LLM summarisation (Ollama) | ✅ | ❌ | — (Scribe advantage) |
| Manual summary regeneration | ✅ | ❓ | — |
| Hybrid human + AI notes (user writes, AI fills in) | ❌ | ✅ | [ ] Add in-meeting scratchpad that merges with transcript |
| Visual distinction: user text vs AI text | ❌ | ✅ (gray = AI, black = user) | [ ] Render user-authored vs AI portions differently |
| Meeting templates (sales call, 1:1, standup, etc.) | ❌ | ✅ (~29 built-in) | [ ] Implement per-meeting-type prompt templates in settings |
| "Recipes" – slash-command AI transforms (Coach Me, Write a Brief, etc.) | ❌ | ✅ | [ ] Add `/recipe` command system in RecordingDetail |
| Edit notes by natural language request ("make this shorter") | ❌ | ✅ | [ ] Add AI edit command in RecordingDetail |
| Chat with notes / Q&A over transcript | ❌ | ✅ | [ ] Add chat panel backed by local LLM / OpenAI-compat |
| Cross-meeting Q&A (query weeks of notes at once) | ❌ | ✅ | [ ] Vector-embed transcripts; add semantic search |
| Glance at previous notes mid-meeting | ❌ | ✅ | [ ] Allow navigation to past recordings while recording active |
| Action item / follow-up extraction | ❌ | ✅ (via templates) | [ ] Add action-item extraction to summary prompt |
| Custom vocabulary / internal jargon for transcription accuracy | ❌ | ✅ | [ ] Add user-defined vocabulary hints passed to ASR |
| AI coaching ("how did you show up in this meeting?") | ❌ | ✅ (Coach Me recipe) | [ ] Add as a built-in recipe once recipe system exists |

---

## 3. Storage & Privacy

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Local-only storage (no cloud) | ✅ | ❌ (AWS cloud-synced) | — (Scribe advantage) |
| Audio never persisted to disk | ✅ | ✅ (transcribed then discarded) | — |
| SQLite text storage | ✅ | ❌ (proprietary cloud DB) | — |
| User-controlled data directory | ✅ | ❌ | — (Scribe advantage) |
| Works fully offline | ✅ | ❌ (requires internet) | — (Scribe advantage) |
| SOC 2 Type 2 certification | ❌ | ✅ (since July 2025) | ❓ Relevant if Scribe adds cloud mode |
| GDPR compliant | ❌ (self-hosted; N/A) | ✅ | ❓ |
| HIPAA compliant | ❌ | ❌ (Granola not HIPAA either) | — |
| Organisation-wide AI training opt-out | ❌ | ✅ (enterprise tier) | ❓ N/A if always local |
| SSO / enterprise auth | ❌ | ✅ (enterprise) | ❓ Only needed if multi-user |
| End-to-end encryption at rest | ❌ | ✅ (AWS encrypted at rest) | [ ] Encrypt SQLite at rest (SQLCipher) |
| Daily encrypted backups | ❌ | ✅ (AWS) | ❓ User's responsibility when local |

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
| Export to Markdown | 🚧 (planned v0.2) | ❌ (copy-paste only) | [ ] Implement `export_recording(id, Markdown)` — Scribe advantage once shipped |
| Export to PDF | 🚧 (planned v0.2) | ❌ (copy-paste only) | [ ] Implement `export_recording(id, PDF)` — Scribe advantage once shipped |
| Copy transcript / summary to clipboard | ❌ | ✅ (primary export path) | [ ] Add copy buttons to RecordingDetail |
| Share notes via link (web-accessible) | ❌ | ✅ | ❓ Conflicts with local-only design |
| Non-Granola users can chat with shared note | ❌ | ✅ | ❓ N/A — no server |
| Export to Notion | ❌ | ✅ | [ ] Add Notion export action (requires user API key) |
| Send to Slack channel | ❌ | ✅ | [ ] Add Slack webhook action (optional) |

---

## 6. Calendar & Meeting Context

| Feature | Scribe | Granola | Todo |
|---------|--------|---------|------|
| Google Calendar integration | ❌ | ✅ | [ ] Read local calendar (CalDAV / Google API) to pre-fill meeting title & attendees |
| Outlook / Microsoft 365 calendar | ❌ | ✅ (added 2026) | [ ] Same — CalDAV may cover both |
| Auto-detect upcoming meeting and open notepad | ❌ | ✅ | [ ] Watch calendar; prompt user N minutes before event |
| Auto-attach notes to calendar event | ❌ | ✅ | [ ] Link recordings to calendar event ID |
| Meeting agenda pre-loaded before call | ❌ | ✅ | [ ] Pull agenda from calendar event description |
| Attendee list from calendar event | ❌ | ✅ | [ ] Store attendees in DB; show in RecordingDetail |
| Pre-meeting collaboration (drop agenda items before call) | ❌ | ✅ | [ ] Pre-meeting note editor linked to upcoming event |

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
6. **SQLite encryption** (SQLCipher): Should the local DB be encrypted at rest? Granola encrypts on AWS; Scribe users own the machine.
7. **Windows support**: Granola supports Windows; Scribe currently does not. Is this a target platform?
8. **Dark mode**: Does Scribe's current Tauri/React UI respect system theme?
9. **Built-in contact tracker**: Granola has a "People & Companies" view (who I met, past notes by person). Wanted in Scribe?
10. **MCP server**: Expose Scribe's local recordings via MCP so Claude / ChatGPT can query them. This seems like a quick win given Granola launched MCP in Feb 2026 and it's a differentiator.
11. **Custom vocabulary**: Granola lets users add internal jargon / product names for better ASR accuracy. Worth adding to Scribe's ASR config?
12. **In-meeting scratchpad (hybrid notes)**: Granola's defining UX — user types bullets mid-meeting, AI merges them with transcript after. Is this the UX direction for Scribe, or keep post-hoc summary only?

---

## Prioritised todo summary (parallelisable work)

### Group A – Low-hanging fruit (no architecture changes)

These can be worked in parallel:

- [ ] Copy-to-clipboard buttons for transcript and summary (Granola's primary export path)
- [ ] SQLite FTS5 full-text search across recordings
- [ ] Implement recording export: Markdown + PDF (already planned v0.2; Scribe advantage — Granola has no native export)
- [ ] Add pagination to `list_recordings` (TD-005)
- [ ] Global keyboard shortcut for start/stop (already planned)
- [ ] Manual audio device picker (TD-003)
- [ ] Action item extraction in summary prompt
- [ ] Dark/light mode verification and fix
- [ ] Visual distinction: render AI-generated portions differently from user-written text

### Group B – Medium effort (self-contained features)

These can be worked in parallel:

- [ ] Per-meeting prompt templates (sales call, 1:1, standup, etc. — ~29 in Granola)
- [ ] Slash-command recipe system (`/coach-me`, `/write-brief`, etc.) in RecordingDetail
- [ ] AI edit command: "make this shorter / more formal"
- [ ] Custom vocabulary / internal jargon setting passed to ASR
- [ ] Notion export action (user provides API key in settings)
- [ ] Slack webhook post-meeting summary (opt-in setting)
- [ ] Configurable generic webhook URL (POST on recording complete)
- [ ] MCP server exposing local recordings (read-only) — quick competitive differentiator
- [ ] Calendar integration: read Google/CalDAV to pre-fill title + attendees
- [ ] Folder / tag organisation in DB + UI
- [ ] "People & Companies" view: group recordings by attendee name (depends on ❓ #9)

### Group C – Larger scope (significant effort)

These can be worked in parallel:

- [ ] Chat / Q&A panel backed by local LLM (in-meeting and post-meeting)
- [ ] Cross-meeting semantic search (vector embeddings + similarity query)
- [ ] Speaker diarisation model integration (note: Granola still requires manual name assignment)
- [ ] Multi-language transcription with auto-detect (multilingual Parakeet/Whisper)
- [ ] Mid-call language switching support (depends on multilingual ASR)
- [ ] In-meeting scratchpad that merges user notes + transcript (Granola's defining UX — see ❓ #12)
- [ ] First-run onboarding wizard (ASR server setup, permissions walkthrough)
- [ ] Windows platform support (WASAPI audio capture)
- [ ] Glance at prior recordings mid-meeting without stopping transcription

### Group D – Needs product decision first (❓ items)

Blocked on answers to Open Questions above:

- [ ] Optional cloud sync / server mode for team collaboration (❓ #1)
- [ ] iOS app (❓ #2)
- [ ] Link-sharing / shareable note URLs (❓ #3)
- [ ] SQLite encryption at rest / SQLCipher (❓ #6)
- [ ] Built-in contact/person tracker (❓ #9)
- [ ] Hybrid in-meeting notes UX (❓ #12)
