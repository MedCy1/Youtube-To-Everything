# Feature Specification: artwork-first interface

Feature Branch: `002-artwork-first-interface`

Created: 2026-11-13

Status: Implemented

Input: Redesign of `y2e-gui`: the video's thumbnail is the backdrop and the interface floats over it.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - An interface that takes the colours of the video (Priority: P1)

As a user, after loading a URL, the video's thumbnail fills the whole window and the interface adopts its dominant hue, with the same contrast whatever the video.

Independent test: load two videos of different colours and check that the accent (timeline, export button) changes hue while staying legible.

Acceptance scenarios:

1. Given a loaded video with its thumbnail, When the image is decoded, Then the accent hue is derived from the dominant colour and the change is animated.
2. Given a thumbnail with no dominant colour (greyscale), When it is decoded, Then the brand hue is kept.

### User Story 2 - Cut by dragging on the timeline (Priority: P1)

As a user, I choose the passage to export by dragging on a waveform. The whole video stays selected until a handle has moved.

Acceptance scenarios:

1. Given a loaded video, When I drag near a bound, Then the closest handle follows the pointer and the selected duration is shown.
2. Given a selection, When I bring it back to the whole video or press "All", Then the cut is disabled.
3. Given the pointer above the timeline, Then a line and the matching time follow it without rebuilding the view.

### User Story 3 - A readable end of export (Priority: P2)

Acceptance scenarios:

1. Given a finished export, Then a halo leaves the button, and a banner shows the file path and offers to open its folder.
2. Given a failed export, Then an error banner appears without blocking the interface.

### User Story 4 - Keyboard and clipboard (Priority: P2)

Acceptance scenarios:

1. Given a YouTube link on the clipboard, When the window comes to the front on the landing view, Then the field is filled and the interface invites the user to press Enter.
2. Given the link field, When I press Enter, Then the video is loaded.
3. Given a loaded video, When I press the arrow keys (outside the link field), Then the end of the passage moves by 1 s (10 s with up and down) and the start moves with Shift, without selecting the timeline first.
4. Given a loaded video, When I navigate with the keyboard, Then every control is reachable, named for screen readers and shows a focus ring. Enter starts the export outside the link field.

## Requirements *(mandatory)*

- FR-001: The accent colour MUST be derived from the thumbnail's dominant hue, at fixed lightness and chroma.
- FR-002: Ambient motion (backdrop drift, artwork breathing, hover line) MUST go through animated values on the render side and MUST NOT rebuild the view.
- FR-003: The URL field MUST remain a single element between the landing view and the top bar.
- FR-004: The final file path MUST come from `yt-dlp` (`--print after_move:filepath`), never from parsing its log.
- FR-005: All non-graphical logic (hue extraction) MUST be covered by a deterministic unit test.
- FR-006: Only a single link to a known YouTube domain MUST be taken from the clipboard; any other content is ignored.
- FR-007: Tool errors MUST be translated into one clear sentence. The technical detail only appears when the error is not recognised.

## Success Criteria *(mandatory)*

- SC-001: Text and controls stay at 14 px or larger and legible on any thumbnail.
- SC-002: No heavy work runs on the interface thread.

## Assumptions

- Decoding the thumbnail and computing its hue happen off the interface thread.
- Export formats and their rules stay those of specification 001.
