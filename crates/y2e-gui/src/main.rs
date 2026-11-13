mod ambient;
mod artwork;
mod gfx;
mod state;
mod theme;
mod tint;
mod view;

use crossbeam_channel::unbounded;
use rune::{App, AppSpec};
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
