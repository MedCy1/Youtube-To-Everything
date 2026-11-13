# Feature Specification: downloader and precise cut

Feature Branch: `001-downloader-snipper`

Created: 2026-11-12

Status: Implemented

Input: A responsive video downloader for YouTube to Everything: automatic metadata, a time cut so that only the chosen segment is downloaded, and presets (maximum quality, Discord compression under 25 MB, podcast audio, GIF), with a Rune interface.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fast download with automatic detection (Priority: P1)

As a user, I paste a YouTube URL (or it is picked up from my clipboard) and I see the title, author, duration and thumbnail within two seconds. I export, and the media is fetched in its best quality with a progress gauge.

Why this priority: without this flow the application is of no use, and it has to respond quickly.

Independent test: paste a valid YouTube URL, export, and check that a playable MP4 file appears in the output folder while the progress gauge advances.

Acceptance scenarios:

1. Given the application is open and a YouTube URL is on the clipboard, When the user focuses the field or pastes, Then the metadata (title, duration, author, thumbnail) appears in under 2 seconds.
2. Given a video is loaded, When the user exports in maximum quality, Then progress is shown as a continuous percentage and the file is in the destination folder at the end.
3. Given an invalid URL or a private or deleted video, When the user tries to load the metadata, Then a clear, non-blocking error message is shown.

---

### User Story 2 - Time cut without a full download (Snipper) (Priority: P2)

As a user who wants a 30-second passage from a 2-hour video or live stream, I set a start point (for example 01:14:00) and an end point (01:14:30). The system downloads only that segment, not the rest of the stream, which saves time and bandwidth.

Why this priority: other downloaders fetch the whole file. Here a fragment comes down without gigabytes of unused data.

Independent test: take a video longer than one hour and request a 30-second segment. The download should take a few seconds and the final file should last exactly 30 seconds.

Acceptance scenarios:

1. Given an inspected video, When the user enables the cut and adjusts the start and end points, Then the valid range and the resulting duration are displayed.
2. Given a configured cut, When the user starts the export, Then only the specified segment is extracted and written to disk.
3. Given inconsistent bounds (start after end, or end beyond the video), When the user tries to confirm, Then the interface prevents the export and highlights the problem.

---

### User Story 3 - Format presets (Priority: P3)

As a user with different needs (sharing a clip on Discord, listening to a podcast on the way to work, or making a meme), I pick a target preset in one click:

- Discord: automatic compression under the 25 MB limit.
- Audio: sound extracted to MP3 with cover art and metadata.
- GIF: a short, light loop.

Why this priority: this is the "To Everything" promise, without asking the user to tune codec settings.

Independent test: try each preset on a test video. Discord output is under 25 MB, the audio preset gives a valid `.mp3`, and the GIF preset gives a working `.gif`.

Acceptance scenarios:

1. Given a selected clip, When the user picks the Discord preset, Then the file never exceeds 24.5 MB while staying as legible as possible.
2. Given a loaded video, When the user picks Audio, Then a light audio file is extracted with the title and author filled in.
3. Given a short selection (under 10 seconds), When the user picks GIF, Then a silent animated image is produced.

---

### Edge cases

- The network connection drops during a download. The system reports the interruption, removes corrupted partial files and allows a clean retry.
- Available disk space is insufficient. An error notification warns the user before the system runs out of room.
- The URL is a live stream in progress. The system detects it and either records from the start or declines with an explanation.

## Requirements *(mandatory)*

### Functional requirements

- FR-001: The system MUST extract and present the metadata (title, creator, duration in seconds, thumbnail URL) from any standard or short (`youtu.be`) YouTube URL.
- FR-002: The system MUST allow manual entry of the URL as well as detection from the clipboard.
- FR-003: The system MUST support downloading arbitrary time ranges `[T_start, T_end]` without first downloading the full stream.
- FR-004: The system MUST provide at least 4 export modes: maximum quality, share (< 25 MB), audio only (MP3) and animated image (GIF).
- FR-005: The system MUST emit a continuous stream of progress notifications giving the percentage done, the average speed and the estimated time remaining.
- FR-006: The system MUST save the final files in a configurable destination folder with a cleaned, safe file name.
- FR-007: The system MUST run asynchronously and never freeze the user interface during transfers or transcoding.

### Key entities

- `VideoMetadata`: the inspected remote video (identifier, title, author, total duration, available streams, thumbnail URL).
- `DownloadTask`: a planned operation (target URL, optional time range, chosen preset, output path).
- `ProgressUpdate`: a snapshot of a task's progress (percentage from 0.0 to 100.0, speed, time remaining, current step).

## Success Criteria *(mandatory)*

### Measurable outcomes

- SC-001: Video metadata appears in under 3 seconds on a standard network connection.
- SC-002: Extracting a 30-second clip from a video longer than 60 minutes takes under 10 seconds in total.
- SC-003: 100% of files produced with the Discord preset weigh strictly less than 24.5 MB.
- SC-004: The interface stays responsive at 60 FPS or better for the whole length of a download.

## Assumptions

- `yt-dlp` and `ffmpeg` are available on the machine, or the application installs them itself (Linux).
- The application is designed first for desktop (Linux, macOS, Windows).
- The default output formats are MP4 for video and MP3 for audio, for compatibility with common players.
