# Contributing to y2e

Thanks for your interest. Bug reports, ideas and pull requests are all welcome. This page explains how to get a change merged without surprises.

## Before you start

- Open an issue first for anything beyond a small fix. A short discussion avoids work on something that does not fit the project.
- For a bug, say what you did, what you expected and what happened, with the y2e version and your distribution. The bug report template asks for these.
- Small fixes (a typo, an obvious bug with a clear cause) can go straight to a pull request.

## Setting up

You need a recent stable Rust toolchain, `curl`, and on Linux the `libxkbcommon` development files (`libxkbcommon-dev` on Debian and Ubuntu). `yt-dlp` and `ffmpeg` are needed to try exports. y2e can install its own copies on first launch.

```bash
git clone https://github.com/MedCy1/Youtube-To-Everything.git
cd Youtube-To-Everything
cargo run --release -p y2e-gui
```

The first build compiles the whole GPU stack and takes a few minutes. Later builds are fast.

## Where things live

| Path | What it holds |
| --- | --- |
| `crates/y2e-core` | The engine: yt-dlp and ffmpeg calls, presets, progress parsing, tool installation. It must stay free of any UI dependency. |
| `crates/y2e-gui` | The interface: state, views, GPU effects, colours. |
| `specs/` | Feature specifications. A feature that changes behaviour should update or add one. |
| `packaging/` | Desktop entry, icon and install script. |

The engine and the interface talk through typed events (`CoreEvent`). New behaviour in the engine should reach the interface that way.

## Making a change

1. Fork the repository and create a branch from `main`.
2. Keep the change focused. One idea per pull request is easier to review and to revert.
3. Add a test for logic that does not draw anything (parsers, size estimates, error mapping). Tests must be deterministic and must not need the network.
4. Run the same checks as CI:

   ```bash
   cargo fmt --all --check
   cargo clippy --all --all-targets --locked -- -D warnings
   cargo test --all --locked
   ```

5. If you change what the interface looks like, include a screenshot in the pull request.

### Style

- Format with `rustfmt` and fix every clippy warning. Do not silence one without a comment saying why.
- Comments explain why the code does something, not what it does.
- Text shown to users is plain English, in short sentences. It will be translated later, so avoid building sentences from fragments.
- Never block the interface thread. Anything slow runs on the Tokio runtime and reports back through an event.

### Commit messages

Commits follow [Conventional Commits](https://www.conventionalcommits.org), because they are turned into the release notes.

```
type(scope): short summary in the imperative

Optional body explaining why, wrapped at about 72 characters.
```

Types in use: `feat`, `fix`, `perf`, `refactor`, `docs`, `build`, `ci`, `style`, `test` and `chore`. Common scopes are `core`, `gui` and `scripts`. Only `feat`, `fix`, `perf`, `refactor`, `build`, `ci` and `docs` appear in the release notes, so write their summaries for a reader of the changelog.

## Pull requests

- Fill in the pull request template. CI has to pass before review.
- Expect feedback. Review comments are about the change, not the person, and a request for changes is a normal part of the process.
- Tidy commits are welcome. The maintainer may squash them when merging.

## License of contributions

y2e is released under the [AGPL-3.0-or-later](LICENSE), and the author also offers it under a separate commercial license. To keep that possible, every contribution is made on these terms:

- You wrote the contribution, or you have the right to submit it.
- You keep the copyright on your contribution.
- You license it under the AGPL-3.0-or-later, like the rest of the project.
- You also grant the project owner, MedCy1, a perpetual, worldwide, non-exclusive, royalty-free and irrevocable license to use, modify, sublicense and distribute your contribution under any other terms, including commercial licenses.

By opening a pull request you accept these terms. The pull request template has a checkbox to confirm it. If you cannot agree to them, please open an issue instead of a pull request and describe the change.

## Releasing

Only the maintainer publishes releases.

```bash
scripts/release.sh 0.2.0     # bumps the version, writes CHANGELOG.md, commits and tags
git push --follow-tags       # the tag starts the release workflow
```

The release workflow runs the tests, builds with the `dist` profile, and publishes the archive, its checksum and the release notes. A tag containing a hyphen, such as `v0.2.0-rc1`, is published as a pre-release. The script needs [`git-cliff`](https://git-cliff.org) (`cargo install git-cliff`).
