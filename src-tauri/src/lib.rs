use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{
  utils::config::WebviewUrl,
  webview::{NewWindowResponse, WebviewWindowBuilder},
};

static NOTE_WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let app_handle = app.handle().clone();
      let app_url = "https://notali-app.vercel.app/"
        .parse()
        .expect("valid Notali app URL");

      WebviewWindowBuilder::new(app, "main", WebviewUrl::External(app_url))
        .title("notali")
        .inner_size(1440.0, 900.0)
        .min_inner_size(1000.0, 650.0)
        .center()
        .on_new_window(move |url, features| {
          // Notali already calls window.open(...) with a note URL on double-click.
          // Tauri does not create that browser popup automatically, so we turn
          // each request into a real desktop webview window instead.
          let id = NOTE_WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
          let label = format!("note-{id}");

          let builder = WebviewWindowBuilder::new(
            &app_handle,
            label,
            WebviewUrl::External("about:blank".parse().expect("valid blank URL")),
          )
          .window_features(features)
          .title("notali")
          .on_document_title_changed(|window, title| {
            let _ = window.set_title(&title);
          });

          match builder.build() {
            Ok(window) => NewWindowResponse::Create { window },
            Err(error) => {
              log::error!("failed to create note window for {url}: {error}");
              NewWindowResponse::Deny
            }
          }
        })
        .build()?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
