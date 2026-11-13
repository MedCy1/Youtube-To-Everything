# Quickstart: YouTube to Everything (y2e)

## System requirements

- Linux, macOS or Windows
- Rust 1.80 or later (`cargo`)
- `yt-dlp` and `ffmpeg` on the `PATH`, or installed by the application on first launch (ffmpeg on Linux and Windows; on macOS it comes from Homebrew)
- `curl`

## Main commands

### 1. Build the libraries and executables

```bash
cargo build --release
```

### 2. Start the native Rune interface

```bash
cargo run --bin y2e-gui
```

### 3. Run the unit and integration tests

```bash
cargo test --all
```
