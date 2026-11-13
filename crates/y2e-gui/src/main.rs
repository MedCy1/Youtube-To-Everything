mod ambient;
mod state;
mod theme;
mod tint;
mod view;

use crossbeam_channel::unbounded;
use rune::{App, AppSpec, Pixels};
use std::sync::Arc;
use y2e_core::{CoreEvent, DownloadOptions, Engine};

use crate::state::{GuiAction, State};
use crate::view::app_view;

fn main() {
    let (action_tx, action_rx) = unbounded::<GuiAction>();
    let _ = state::ACTION_TX.set(action_tx);

    let spec = AppSpec {
        fonts: rune_theme_opencode::FONTS,
        families: rune_theme_opencode::FAMILIES,
        ..AppSpec::new(
            "YouTube to Everything // Studio",
            (1140.0, 740.0),
            State::default,
            app_view,
        )
    };

    ambient::start();
    let app = App::new(spec);
    let app_handle = Arc::new(app.handle());
    let handle_for_worker = app_handle.clone();

    // Spawn async background runtime for yt-dlp & ffmpeg pipelines
    std::thread::Builder::new()
        .name("y2e-worker".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to initialize background Tokio runtime");

            rt.block_on(async move {
                while let Ok(action) = action_rx.recv() {
                    let handle = handle_for_worker.clone();
                    match action {
                        GuiAction::InspectMetadata(url) => {
                            tokio::spawn(async move {
                                match Engine::fetch_metadata(&url).await {
                                    Ok(metadata) => {
                                        let thumbnail_url = metadata.thumbnail.clone();
                                        handle.update(move |s, _tx| {
                                            s.on_inspect_success(metadata);
                                        });
                                        if let Some(url) = thumbnail_url {
                                            let preview_handle = handle.clone();
                                            tokio::spawn(async move {
                                                let Ok(output) =
                                                    tokio::process::Command::new("curl")
                                                        .args([
                                                            "--fail",
                                                            "--location",
                                                            "--silent",
                                                            "--max-time",
                                                            "10",
                                                            &url,
                                                        ])
                                                        .output()
                                                        .await
                                                else {
                                                    return;
                                                };
                                                if !output.status.success() {
                                                    return;
                                                }
                                                let bytes = output.stdout;
                                                let preview =
                                                    tokio::task::spawn_blocking(move || {
                                                        let rgba = image::load_from_memory(&bytes)
                                                            .ok()?
                                                            .into_rgba8();
                                                        let hue = tint::dominant_hue(
                                                            &rgba,
                                                            rgba.width(),
                                                            rgba.height(),
                                                        );
                                                        let pixels = Pixels::new();
                                                        pixels.set(
                                                            rgba.width(),
                                                            rgba.height(),
                                                            rgba.into_raw(),
                                                        );
                                                        Some((pixels, hue))
                                                    })
                                                    .await
                                                    .ok()
                                                    .flatten();
                                                if let Some((preview, hue)) = preview {
                                                    preview_handle.update(move |s, tx| {
                                                        tx.animated();
                                                        s.thumbnail = Some(preview);
                                                        s.accent_hue =
                                                            hue.unwrap_or(state::BRAND_HUE);
                                                    });
                                                }
                                            });
                                        }
                                    }
                                    Err(err) => {
                                        handle.update(move |s, _tx| {
                                            s.on_inspect_error(err.to_string());
                                        });
                                    }
                                }
                            });
                        }
                        GuiAction::StartDownload {
                            url,
                            time_range,
                            preset,
                            output_dir,
                        } => {
                            let (event_tx, mut event_rx) =
                                tokio::sync::mpsc::channel::<CoreEvent>(64);
                            let event_handle = handle.clone();

                            // Forward events to Rune UI
                            tokio::spawn(async move {
                                while let Some(ev) = event_rx.recv().await {
                                    let h = event_handle.clone();
                                    h.update(move |s, _tx| {
                                        s.apply_core_event(ev);
                                    });
                                }
                            });

                            let opts = DownloadOptions {
                                url,
                                time_range,
                                preset,
                                output_dir,
                            };

                            tokio::spawn(async move {
                                if let Err(e) = Engine::download(opts, event_tx.clone()).await {
                                    let _ = event_tx.send(CoreEvent::Error(e.to_string())).await;
                                }
                            });
                        }
                    }
                }
            });
        })
        .expect("Failed to spawn background worker thread");

    app.run();
}
