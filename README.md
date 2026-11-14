<div align="center">

<img src="packaging/y2e.svg" alt="y2e logo" width="96" height="96">

# YouTube to Everything

Download a YouTube video, keep only the part you want, and export it in the format you need.

[![CI](https://github.com/MedCy1/Youtube-To-Everything/actions/workflows/ci.yml/badge.svg)](https://github.com/MedCy1/Youtube-To-Everything/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/MedCy1/Youtube-To-Everything?include_prereleases&sort=semver)](https://github.com/MedCy1/Youtube-To-Everything/releases)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-informational)
![Rust](https://img.shields.io/badge/built%20with-Rust-orange)
[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)](LICENSE)

[Installation](#installation) · [Features](#features) · [Keyboard shortcuts](#keyboard-shortcuts) · [Build from source](#build-from-source) · [Contributing](#contributing) · [License](#license)

<br>

<img src="docs/screenshots/studio.jpg" alt="The studio view: the video's thumbnail fills the window, with the title, a waveform timeline and the export controls floating above it" width="860">

</div>

## Overview

y2e (YouTube to Everything) is a desktop application for turning a YouTube video into a file you can actually use: an MP4, an MP3, a clip small enough for Discord, or a GIF. You paste a link, drag across a waveform to choose the passage you care about, pick a format and export. Only the selected segment is downloaded, so a short clip from a long video takes a fraction of the time a full download would.

The interface is native and GPU rendered. The window appears in about 150 ms, the studio view draws nothing while it is at rest, and the colours come from the video's thumbnail.

<div align="center">
<img src="docs/screenshots/landing.jpg" alt="The start screen: a headline, a field to paste a YouTube link and a Load button over a colour gradient" width="640">
</div>

## Features

### Cut before you download

Dragging on the timeline selects the passage to keep. The whole video stays selected until you move a handle, and the arrow keys nudge either end by a second at a time. Because the cut happens at the source, y2e never downloads the part you throw away.

<div align="center">
<img src="docs/screenshots/clip.jpg" alt="A 45 second clip selected on the timeline, with the GIF preset chosen and a suggestion to keep a shorter clip" width="760">
</div>

### Four exports

| Preset | Output | How |
| --- | --- | --- |
| Max quality | MP4 | The best video and audio streams, merged without re-encoding where possible. |
| Audio | MP3 | The audio track alone, at the highest quality setting. |
| Discord | MP4 under 25 MB | H.264 and AAC, with the video bitrate worked out from the length of the clip so the file fits the limit. |
| Loop | GIF | 15 frames per second, 480 pixels wide, with an optimised palette. |

The estimated size is shown before you export. Nothing is ever blocked: a very large GIF is still possible, and y2e only suggests a shorter clip.

### An interface that follows the video

The thumbnail fills the window and the accent colour is derived from its dominant hue, at a fixed lightness so text and controls stay readable whatever the video.

<div align="center">
<img src="docs/screenshots/colors.jpg" alt="Three videos, three accent colours: orange, blue and red" width="860">
</div>

### Built for the keyboard

Every control can be reached with the keyboard, shows a focus ring and is exposed to the system accessibility API (VoiceOver, Narrator and NVDA, Orca), with the clip range, the export progress and errors announced. The cut can be adjusted without touching the mouse. A YouTube link found on the clipboard when the window comes to the front is offered straight away. See the [shortcuts](#keyboard-shortcuts).

### It looks after its own tools

y2e drives `yt-dlp` and `ffmpeg`. It uses the ones on your `PATH`, and if they are missing it offers to install them for you, then checks for newer versions of `yt-dlp` at launch. See [yt-dlp and ffmpeg](#yt-dlp-and-ffmpeg).

### Clear errors

Tool failures are turned into one plain sentence ("This video is not available.", "Could not connect. Check your network."). The technical detail only appears when the error is not recognised.

## Installation

Packages for Linux, macOS and Windows are published on the [Releases](https://github.com/MedCy1/Youtube-To-Everything/releases) page. Each file comes with a `.sha256` checksum.

### Linux

On Debian, Ubuntu and derivatives (Ubuntu 24.04 or Debian 13 and later), install the package:

```bash
sudo apt install ./y2e-vX.Y.Z-linux-amd64.deb
```

On other distributions, use the tarball:

```bash
tar -xzf y2e-vX.Y.Z-linux-x86_64.tar.gz
cd y2e-vX.Y.Z-linux-x86_64
sh install.sh
```

`install.sh` copies the binary to `~/.local/bin` and adds a menu entry and an icon for the current user. Run `sh install.sh --uninstall` to remove them. You need a Wayland or X11 session and a GPU driver with Vulkan or OpenGL support.

### macOS

Open `y2e-vX.Y.Z-macos-universal.dmg` and drag y2e to Applications. The app runs on Apple silicon and Intel Macs, macOS 11 or later. Until the app is signed with an Apple developer certificate, the first launch needs a right-click and Open, or `xattr -dr com.apple.quarantine /Applications/y2e.app`.

### Windows

Run `y2e-vX.Y.Z-windows-x86_64-setup.exe`. It installs for the current user without asking for administrator rights, adds a Start menu entry and an uninstaller. A portable `.zip` with just the executable is published too. Windows SmartScreen may warn about an unknown publisher the first time: choose More info, then Run anyway. Uninstalling keeps the tools y2e downloaded, in `%LOCALAPPDATA%\y2e`; delete that folder to remove them.

## yt-dlp and ffmpeg

y2e relies on [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) to read and fetch videos and on [`ffmpeg`](https://ffmpeg.org) to convert them.

- If both are installed on your system, y2e uses them. On macOS it also looks in the Homebrew folders.
- If one is missing, the start screen says so and offers to install it. On Linux and Windows that downloads about 190 MB and takes about 390 MB on disk; the macOS builds are smaller. Each file is checked against the SHA-256 sum its release publishes before it is installed.
- The tools land in `~/.local/share/y2e/bin` on Linux, `~/Library/Application Support/y2e/bin` on macOS and `%LOCALAPPDATA%\y2e\bin` on Windows.
- On macOS, `ffmpeg` and `ffprobe` come from [Martin Riedl's build server](https://ffmpeg.martin-riedl.de), which publishes static builds for Apple silicon and Intel with a SHA-256 sum for each file. The sum is checked the same way as for the other tools.
- At launch, y2e looks for a newer `yt-dlp` in the background and offers to update it. If your copy comes from a package manager, y2e tells you about the update and leaves the update to you.
- A copy of `ffmpeg` installed by y2e is offered for renewal after four months.

## Keyboard shortcuts

| Key | Action |
| --- | --- |
| Enter (in the link field) | Load the video |
| Tab | Move between controls |
| Enter (elsewhere) | Export |
| Left, Right | Move the end of the clip by 1 second |
| Up, Down | Move the end of the clip by 10 seconds |
| Shift with an arrow | Move the start instead of the end |
| Home | Select the whole video again |
| Escape | Cancel the export in progress |

The arrow keys work as soon as a video is loaded, without selecting the timeline first, except while you are typing in the link field. Exported files are saved in your Downloads folder.

## Build from source

You need a recent stable Rust toolchain. y2e also calls `curl` and `tar`, which ship with macOS and Windows 10 or later. On Linux the build needs the `libxkbcommon` development files (`libxkbcommon-dev` on Debian and Ubuntu).

```bash
git clone https://github.com/MedCy1/Youtube-To-Everything.git
cd Youtube-To-Everything
cargo run --release -p y2e-gui
```

`cargo build --profile dist -p y2e-gui` produces the binary that is shipped in releases. It takes longer to compile and the executable is about 40 percent smaller.

## Project layout

```
crates/
├── y2e-core/   engine: yt-dlp, ffmpeg, presets, progress, tool management (no UI dependency)
└── y2e-gui/    native interface: views, GPU effects, colour palette
packaging/      Linux desktop entry and installer, macOS bundle script, icons
scripts/        release and third-party license scripts
third-party/    licenses of the embedded fonts
specs/          feature specifications
```

The engine and the interface only exchange typed events (`CoreEvent`), so another front end can be added without touching the engine.

The interface is built with [Rune](https://github.com/anomalyco/rune), a GPU UI framework for Rust.

## Contributing

Bug reports, ideas and pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) first: it covers the setup, the checks CI runs, the commit conventions and the terms contributions are made under. Open an issue before starting on anything larger than a small fix.

## Roadmap

- A settings page (output folder, default preset, motion preferences).
- A language switch, with French as the first translation.
- Chapters, so a video can be cut by section.

## Known limitations

- When a cut is selected, `yt-dlp` hands the download to `ffmpeg`, which does not report progress. The export button stays at 0 % until the file is ready.
- Screen reader support is built in but has not been tried with VoiceOver, Narrator, NVDA or Orca yet. Reports are welcome.
- Cancelling a cut export stops `yt-dlp`, but on Windows the `ffmpeg` it started may keep running for a few seconds.
- Builds and tests run on Linux, macOS and Windows in CI, but the interface has so far been used by hand on Linux only. Please report anything that looks wrong on the other two.

## License

y2e is free software, released under the [GNU Affero General Public License, version 3 or later](LICENSE). You can use, study, modify and share it, including commercially. If you distribute a modified version, or run one as a network service, you must release your source code under the same license and keep the copyright notices.

Copyright (c) 2026 MedCy1.

### Commercial license

If the AGPL does not fit your use, for example because you want to include y2e or its engine in a closed-source product, a separate commercial license is available from the author. [Open an issue](https://github.com/MedCy1/Youtube-To-Everything/issues/new?template=commercial_license.yml) with the commercial license template. Issues are public, so keep confidential details out of it: the author will reply to arrange a private way to continue.

Third-party dependencies keep their own licenses, mostly MIT and Apache-2.0. Each release archive includes `THIRD-PARTY-LICENSES.txt` with their notices and the licenses of the embedded fonts.

## Responsible use

y2e is meant for videos you have the right to download, such as your own uploads, freely licensed work and content you have permission to use. You are responsible for following YouTube's terms of service and the copyright law that applies to you.

## Acknowledgements

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) and [FFmpeg](https://ffmpeg.org) do the real work.
- Prebuilt FFmpeg binaries come from [yt-dlp/FFmpeg-Builds](https://github.com/yt-dlp/FFmpeg-Builds).
- The screenshots show "Big Buck Bunny", "Sintel" and "Tears of Steel", open movies by the [Blender Foundation](https://studio.blender.org) released under the Creative Commons Attribution license, and "Me at the zoo" by jawed.
