# Phase 0 Research: y2e engine and Rune integration

## 1. yt-dlp: metadata extraction and streaming

### Metadata extraction

The inspection command:

```bash
yt-dlp --dump-json --skip-download --no-playlist "<URL>"
```

It writes a JSON object to standard output containing:

- `id`, `title`, `uploader`, `duration` (in seconds)
- `thumbnail` (URL of the best thumbnail)
- `formats`: the array of video and audio streams
- `chapters`: a list of `{ "start_time": f64, "end_time": f64, "title": String }` objects

### Time cut without a full download

`yt-dlp` supports the `--download-sections` syntax, combined with HTTP Range requests:

```bash
yt-dlp --download-sections "*01:15:00-01:15:30" --force-keyframes-at-cuts "<URL>" -o "output.%(ext)s"
```

YouTube only sends the data segments matching the requested timestamps, so a 30-second segment downloads in a few seconds even from a 3-hour video.

### Progress tracking

`yt-dlp` supports a custom progress format through `--newline` and `--progress-template`:

```bash
yt-dlp --newline --progress-template "DOWNLOAD_PROGRESS:%(progress._percent_str)s:%(progress._speed_str)s:%(progress._eta_str)s"
```

The format is stable, so parsing it in Rust is plain string splitting.

---

## 2. FFmpeg encoding and transcoding presets

### Discord preset (< 25 MB)

To make a video fit under the 25 MB limit (strict 24.5 MB target, for a safety margin):

1. Overall bitrate:

   $$\text{Total Bitrate (kbps)} = \frac{24.5 \times 8192}{\text{Duration (seconds)}}$$

2. Allocation: audio = 96 kbps (AAC), video = $\text{Total Bitrate} - 96$.
3. FFmpeg encoding:

   ```bash
   ffmpeg -i input.mp4 -c:v libx264 -b:v <video_bitrate>k -maxrate <video_bitrate>k -bufsize <2*video_bitrate>k -c:a aac -b:a 96k output_discord.mp4
   ```

### Audio preset

Extract the audio track alone, converted to MP3 or M4A with the thumbnail embedded:

```bash
yt-dlp -x --audio-format mp3 --audio-quality 0 --embed-thumbnail "<URL>" -o "output.%(ext)s"
```

### GIF preset

An optimised palette, generated in two passes within one FFmpeg command line:

```bash
ffmpeg -i input.mp4 -vf "fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" output.gif
```

---

## 3. Rune architecture and concurrency

- Reactive model: the function `view(s: &State, cx: &mut Cx<State>) -> El` rebuilds the GPU element tree on every mutation of `State`.
- Async bridge: the `yt-dlp` and `ffmpeg` calls run in a `tokio` runtime. A `std::sync::mpsc` channel carries actions from the interface to the runtime, and the application handle (`AppHandle::update`) brings results back into Rune's state.
