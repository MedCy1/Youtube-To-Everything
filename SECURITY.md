# Security policy

## Supported versions

Only the latest release receives security fixes.

## Reporting a vulnerability

Please do not open a public issue for a security problem.

Report it privately through GitHub: open the [Security tab](https://github.com/MedCy1/Youtube-To-Everything/security/advisories/new) of the repository and choose "Report a vulnerability". Include what you found, the steps to reproduce it, the y2e version and your system.

You can expect an answer within a week. If the report is confirmed, a fix is released as soon as it is ready and the report is credited to you, unless you prefer to stay anonymous.

## What is in scope

y2e downloads and runs programs on the user's machine, so these matter most:

- The installation of `yt-dlp` and `ffmpeg`: the checks on what is downloaded, and where it is written.
- The handling of text coming from the network or from a video's metadata (titles, links, file names).
- The release packages and the workflows that build them.

Problems in `yt-dlp`, `ffmpeg` or the Rune framework belong to those projects.
