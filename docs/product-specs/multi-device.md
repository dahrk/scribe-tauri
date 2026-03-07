# Product Spec: Multi-Device Audio (Future)

## Status: Future consideration

## Problem

Some users have multiple microphones or output devices (e.g. a USB headset
and a built-in mic simultaneously for a room conference). The current heuristic
(default input device + monitor/loopback detection) may pick the wrong device.

## Proposed solution

Settings UI: device picker dropdowns for "Microphone device" and
"System audio device" populated via a new `list_audio_devices` Tauri command.

## Implementation notes

- `cpal::Host::input_devices()` already enumerates available devices.
- New Tauri command: `list_audio_devices() -> Vec<AudioDeviceInfo>`.
- `AudioDeviceInfo`: `{ id: String, name: String, is_monitor: bool }`.
- Persist the selected device IDs in `Settings`.
- Fall back to default device if saved ID is no longer available.
