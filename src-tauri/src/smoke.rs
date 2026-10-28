//! Test d’intégration opt-in, uniquement dans les builds de développement.
use super::*;
use std::{sync::mpsc, time::Duration};
use tauri::Listener;
pub fn run(app: tauri::AppHandle, base: String) {
    let (tx, rx) = mpsc::channel();
    app.listen("page", move |event| {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            if let Some(t) = v["title"].as_str() {
                let _ = tx.send(t.to_string());
            }
        }
    });
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let shell = app.get_webview("main").ok_or("shell absent")?;
            tauri::async_runtime::block_on(open_tab(
                app.clone(),
                shell.clone(),
                "tab-smoke".into(),
                format!("{base}/one"),
                240.,
                80.,
                800.,
                600.,
            ))?;
            let wait = |title: &str| -> Result<(), String> {
                for _ in 0..12 {
                    if rx
                        .recv_timeout(Duration::from_secs(5))
                        .map_err(|e| e.to_string())?
                        == title
                    {
                        return Ok(());
                    }
                }
                Err(format!("Titre attendu : {title}"))
            };
            wait("NOVA smoke one")?;
            tauri::async_runtime::block_on(tab_action(
                app.clone(),
                shell.clone(),
                "tab-smoke".into(),
                "navigate".into(),
                Some(format!("{base}/two")),
            ))?;
            wait("NOVA smoke two")?;
            tauri::async_runtime::block_on(tab_action(
                app.clone(),
                shell.clone(),
                "tab-smoke".into(),
                "back".into(),
                None,
            ))?;
            wait("NOVA smoke one")?;
            let remote = app.get_webview("tab-smoke").ok_or("page absente")?;
            assert!(write_data(app.clone(), remote, "settings".into(), "{}".into()).is_err());
            write_data(
                app.clone(),
                shell.clone(),
                "bookmark".into(),
                serde_json::json!({"url":format!("{base}/one"),"title":"Smoke"}).to_string(),
            )?;
            let marks = read_data(app.clone(), shell.clone(), "bookmarks".into())?;
            assert!(marks
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["title"] == "Smoke"));
            write_data(
                app.clone(),
                shell.clone(),
                "unbookmark".into(),
                format!("{base}/one"),
            )?;
            assert!(!read_data(app.clone(), shell.clone(), "history".into())?
                .as_array()
                .unwrap()
                .is_empty());
            tauri::async_runtime::block_on(tab_action(
                app.clone(),
                shell,
                "tab-smoke".into(),
                "close".into(),
                None,
            ))?;
            assert!(app.get_webview("tab-smoke").is_none());
            println!(
                "NOVA_SMOKE_OK: navigation, JavaScript, retour, SQLite, isolation IPC, fermeture"
            );
            Ok(())
        })();
        match result {
            Ok(()) => app.exit(0),
            Err(e) => {
                eprintln!("NOVA_SMOKE_FAILED: {e}");
                app.exit(1);
            }
        }
    });
}
