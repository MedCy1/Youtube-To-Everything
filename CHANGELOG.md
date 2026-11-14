# Changelog

## v0.4.2 (2026-11-14)


### Features

- Show an export as stages, each with its own progress

### Bug fixes

- gui: Say an export has started and forget the last one's figures
## v0.4.1 (2026-11-14)


### Features

- core: Show the progress of the Discord and GIF encoding

### Bug fixes

- core: Show the progress of an export cut from a video
- Aim for Discord's 20 MB limit instead of 25 MB

### Documentation

- Drop the cut progress limitation from the README
## v0.4.0 (2026-11-14)


### Features

- Translate the interface to French, with a language setting
- Cut a video by chapter

### Bug fixes

- gui: Keep the focus ring whole in the chapter list and make the cards opaque
- gui: Show the blur behind the settings and chapter cards from the first frame
- gui: Keep the blur behind the cards in the studio

### Internal

- gui: Drop the video loaded chip to free a blur for the cards
## v0.3.0 (2026-11-14)


### Features

- Tell the user when a newer release of the app exists
- Add a settings window

### Bug fixes

- gui: Fall back to the home folder when the system has no Downloads folder

### Build and CI

- Give the workflows read-only access unless a job needs more
- Move the GitHub Actions to their Node 24 releases
- Update dirs to 7 and sha2 to 0.11

### Documentation

- Add a security policy and complete the README, and add repository housekeeping files
## v0.2.0 (2026-11-14)


### Features

- Expose the interface to screen readers
- Cancel an export, and keep a crash log
- Download ffmpeg for macOS instead of asking for Homebrew

### Build and CI

- Ship installers for Windows, macOS and Debian
- Keep Git Bash from rewriting the Inno Setup options

### Documentation

- Describe the installers and how to sign the macOS app
## v0.1.0 (2026-11-14)


### Features

- Implement y2e-core engine and Rune native GPU interface
- gui: Fetch and decode the source artwork
- gui: Artwork-first interface with a glass export dock
- gui: Tint the interface with the hue of the source artwork
- gui: Ambient motion, shared-element entrance and timeline scrubbing
- gui: Celebrate a finished export
- Estimate the output size and steer heavy GIFs without blocking them
- gui: Draw the landing aurora and export glow with rune-fx
- gui: Submit with Return and explain errors in plain language
- gui: Keyboard access and clipboard link detection
- gui: Keyboard control of the cut, tighter dock, and a README
- gui: Say up front when yt-dlp or ffmpeg is missing
- Install yt-dlp and ffmpeg from the landing view when they are missing
- Offer updates for yt-dlp and ffmpeg on the landing view
- Switch the interface and release notes to English
- Make the app run on Linux, macOS and Windows
- Install ffmpeg through Homebrew on macOS when it is available
- Show the progress of the tool installation
- Show Homebrew's download percentage during the ffmpeg install

### Bug fixes

- Handle null chapters in metadata and isolate inspect and download errors
- core: Read the final output path from yt-dlp
- gui: Reveal the source only once its artwork is ready
- gui: Keep the layout still during an export and adapt the scrim to the artwork
- gui: Polish the landing errors and loading, drop the artwork zoom
- gui: Recognise truncated links and keep error text inside its card
- core: Report ffmpeg and yt-dlp failures instead of hiding them
- core: Size the Discord export from the real length of the file
- scripts: Make release.sh work when the version is already set

### Performance

- gui: Let the export aura rest unless it is hovered or exporting

### Internal

- Drop unused dependencies and dead code, ease progress updates
- gui: Split the view into landing, studio, timeline and dock

### Build and CI

- Add a dist profile with full LTO and stripped symbols
- Add continuous integration and a tagged release workflow
- Share package metadata and pin Rune, reorganise the README
- Ship third-party license notices with each release
- Allow a manual rehearsal of the release workflow
- Test on three systems and publish macOS and Windows packages

### Documentation

- Add core downloader and precision snipper specifications
- Complete implementation tasks for core downloader and Rune GUI
- specs: Describe the artwork-first interface
- Relax the startup target to 200 ms
- Plainer wording, current specs and no decorative markup
- Translate the specifications to English
- Rewrite the README in English with new screenshots
- Add a contributing guide and issue and pull request templates
- Cover macOS and Windows in the README and contributing guide

