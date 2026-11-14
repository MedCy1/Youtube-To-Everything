// No console window behind the interface on Windows.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod artwork;
mod crash;
mod gfx;
mod link;
mod report;
mod state;
mod theme;
mod tint;
mod view;

use rune::{App, AppSpec};
use std::sync::Arc;
use y2e_core::{CoreEvent, DownloadOptions, Engine};

use crate::state::{GuiAction, State};
use crate::view::app_view;

fn main() {
    crash::keep_a_log();
    let (action_tx, action_rx) = std::sync::mpsc::channel::<GuiAction>();
    if y2e_core::tools::missing().is_empty() {
        let _ = action_tx.send(GuiAction::CheckUpdates);
    }
    let _ = action_tx.send(GuiAction::CheckAppUpdate);
    let _ = state::ACTION_TX.set(action_tx);

    let spec = AppSpec {
        fonts: rune_theme_opencode::FONTS,
        families: rune_theme_opencode::FAMILIES,
        // The cut answers the arrow keys whenever no field is being edited.
        on_key: Some(|s: &mut State, key, _| s.timeline_key(key)),
        ..AppSpec::new(
            "YouTube to Everything // Studio",
            (1140.0, 740.0),
            State::default,
            app_view,
        )
    };

    let app = App::new(spec).compositor(gfx::StudioGfx::default());
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
                // The tasks of the export in progress, so that it can be stopped.
                let mut export: Vec<tokio::task::JoinHandle<()>> = Vec::new();
                while let Ok(action) = action_rx.recv() {
                    let handle = handle_for_worker.clone();
                    match action {
                        GuiAction::InspectMetadata(url) => {
                            tokio::spawn(async move {
                                match Engine::fetch_metadata(&url).await {
                                    Ok(metadata) => {
                                        // The interface only appears once the artwork is
                                        // decoded, so its colours are final when it does.
                                        let artwork = match metadata.thumbnail.clone() {
                                            Some(url) => artwork::fetch(&url).await,
                                            None => None,
                                        };
                                        handle.update(move |s, tx| {
                                            tx.animated();
                                            // Leave the URL field so the arrow keys reach the timeline.
                                            tx.blur();
                                            s.on_inspect_success(metadata, artwork);
                                        });
                                    }
                                    Err(err) => {
                                        handle.update(move |s, _tx| {
                                            s.on_inspect_error(err.to_string());
                                        });
                                    }
                                }
                            });
                        }
                        GuiAction::CheckUpdates => {
                            tokio::spawn(async move {
                                let updates = y2e_core::tools::updates().await;
                                handle.update(move |s, tx| {
                                    if !updates.is_empty() {
                                        tx.animated();
                                    }
                                    s.updates = updates;
                                });
                            });
                        }
                        GuiAction::CheckAppUpdate => {
                            tokio::spawn(async move {
                                if let Some(update) =
                                    y2e_core::app_update::check(env!("CARGO_PKG_VERSION")).await
                                {
                                    handle.update(move |s, tx| {
                                        tx.animated();
                                        s.app_update = Some(update);
                                    });
                                }
                            });
                        }
                        GuiAction::InstallTools(tools) => {
                            tokio::spawn(async move {
                                let progress = handle.clone();
                                let result = y2e_core::tools::install(&tools, move |phase| {
                                    progress.update(move |s, _| {
                                        s.installing = Some(phase.text);
                                        s.install_fraction = phase.fraction;
                                    });
                                })
                                .await;
                                handle.update(move |s, tx| {
                                    tx.animated();
                                    s.finish_install(result.map_err(|e| e.to_string()));
                                    if s.tools_ready() {
                                        tx.focus("url_input", false);
                                    }
                                });
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
                            let forwarder = tokio::spawn(async move {
                                // yt-dlp reports ~100 times a second; the bar eases between
                                // targets, so a handful of updates per second looks the same.
                                let mut last = std::time::Instant::now();
                                while let Some(ev) = event_rx.recv().await {
                                    if matches!(ev, CoreEvent::Progress(_)) {
                                        if last.elapsed() < std::time::Duration::from_millis(125) {
                                            continue;
                                        }
                                        last = std::time::Instant::now();
                                    }
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

                            let worker = tokio::spawn(async move {
                                if let Err(e) = Engine::download(opts, event_tx.clone()).await {
                                    let _ = event_tx.send(CoreEvent::Error(e.to_string())).await;
                                }
                            });
                            export = vec![forwarder, worker];
                        }
                        GuiAction::CancelDownload => {
                            for task in export.drain(..) {
                                task.abort();
                            }
                        }
                    }
                }
            });
        })
        .expect("Failed to spawn background worker thread");

    app.run();
}
