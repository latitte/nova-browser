#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::{
    webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
    Emitter, Manager, Webview, WebviewUrl,
};

fn trusted(view: &Webview) -> Result<(), String> {
    if view.label() == "main" {
        Ok(())
    } else {
        Err("Accès natif refusé".into())
    }
}
fn http_url(raw: &str) -> Result<url::Url, String> {
    let u = url::Url::parse(raw).map_err(|_| "URL invalide")?;
    if !matches!(u.scheme(), "https" | "http")
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
    {
        return Err("Seules les adresses HTTP(S) sans identifiants sont autorisées".into());
    }
    Ok(u)
}
fn tab_id(id: &str) -> Result<(), String> {
    if id.starts_with("tab-")
        && id.len() < 64
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        Ok(())
    } else {
        Err("Onglet invalide".into())
    }
}
fn db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let p = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    let c = Connection::open(p.join("nova.sqlite")).map_err(|e| e.to_string())?;
    migrate(&c)?;
    Ok(c)
}
fn migrate(c: &Connection) -> Result<(), String> {
    c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
        .map_err(|e| e.to_string())?;
    let version: i64 = c
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if version > 1 {
        return Err("Cette base de données nécessite une version plus récente de NOVA".into());
    }
    if version == 0 {
        c.execute_batch("BEGIN; CREATE TABLE history(id INTEGER PRIMARY KEY, url TEXT NOT NULL, title TEXT NOT NULL, visited INTEGER NOT NULL DEFAULT (unixepoch()));
        CREATE INDEX history_visited ON history(visited DESC);
        CREATE TABLE bookmarks(url TEXT PRIMARY KEY, title TEXT NOT NULL);
        CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
        PRAGMA user_version=1; COMMIT;").map_err(|e|e.to_string())?;
    }
    Ok(())
}
#[derive(Clone, Serialize)]
struct Page {
    id: String,
    url: String,
    title: Option<String>,
    loading: bool,
}
#[derive(Serialize, Deserialize)]
struct Entry {
    url: String,
    title: String,
}
#[tauri::command]
async fn open_tab(
    app: tauri::AppHandle,
    webview: Webview,
    id: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    trusted(&webview)?;
    tab_id(&id)?;
    let url = http_url(&url)?;
    if app.get_webview(&id).is_some() {
        return Err("Cet onglet existe déjà".into());
    }
    let window = app.get_window("main").ok_or("Fenêtre absente")?;
    let nav_app = app.clone();
    let nav_id = id.clone();
    let title_app = app.clone();
    let title_id = id.clone();
    let popup_app = app.clone();
    let builder = WebviewBuilder::new(&id, WebviewUrl::External(url))
        .on_navigation(move |url| {
            let allowed = http_url(url.as_str()).is_ok();
            if allowed { let _ = nav_app.emit_to("main", "page", Page { id: nav_id.clone(), url: url.to_string(), title: None, loading: true }); }
            allowed
        })
        .on_document_title_changed(move |view, title| {
            if let Ok(url) = view.url() { let _ = title_app.emit_to("main", "page", Page { id: title_id.clone(), url: url.to_string(), title: Some(title), loading: false }); }
        })
        .on_page_load(move |view, payload| {
            let loading = matches!(payload.event(), PageLoadEvent::Started);
            let _ = view.app_handle().emit_to("main", "page", Page { id: view.label().into(), url: payload.url().to_string(), title: None, loading });
            if !loading {
                if let Ok(c) = db(view.app_handle()) { let _ = c.execute("INSERT INTO history(url,title) VALUES(?1,?2)", params![payload.url().as_str(), payload.url().host_str().unwrap_or("Page")]); }
            }
        })
        .on_new_window(move |url, _| {
            if http_url(url.as_str()).is_ok() { let _ = popup_app.emit_to("main", "popup", url.to_string()); }
            NewWindowResponse::Deny
        })
        .on_permission_request(|view, _| { let _ = view.app_handle().emit_to("main", "notice", "Permission refusée : la gestion des autorisations par site n’est pas encore disponible."); tauri::webview::PermissionResponse::Deny })
        .on_download(|view, _| { let _ = view.app_handle().emit_to("main", "notice", "Les téléchargements ne sont pas encore disponibles dans ce MVP."); false });
    window
        .add_child(
            builder,
            tauri::LogicalPosition::new(x.max(0.), y.max(0.)),
            tauri::LogicalSize::new(width.max(1.), height.max(1.)),
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
async fn tab_action(
    app: tauri::AppHandle,
    webview: Webview,
    id: String,
    action: String,
    value: Option<String>,
) -> Result<(), String> {
    trusted(&webview)?;
    tab_id(&id)?;
    let v = app.get_webview(&id).ok_or("Onglet fermé")?;
    let r = match action.as_str() {
        "navigate" => v.navigate(http_url(value.as_deref().unwrap_or(""))?),
        "back" => v.eval("history.back()"),
        "forward" => v.eval("history.forward()"),
        "reload" => v.reload(),
        "stop" => v.eval("window.stop()"),
        "close" => v.close(),
        "hide" => v.hide(),
        "show" => v.show(),
        "zoom" => {
            let scale = value
                .and_then(|s| s.parse::<f64>().ok())
                .filter(|s| s.is_finite() && *s >= 0.5 && *s <= 2.)
                .ok_or("Zoom invalide")?;
            v.set_zoom(scale)
        }
        _ => return Err("Action inconnue".into()),
    };
    r.map_err(|e| e.to_string())
}
#[tauri::command]
async fn layout(
    app: tauri::AppHandle,
    webview: Webview,
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    trusted(&webview)?;
    tab_id(&id)?;
    if let Some(v) = app.get_webview(&id) {
        v.set_position(tauri::LogicalPosition::new(x.max(0.), y.max(0.)))
            .map_err(|e| e.to_string())?;
        v.set_size(tauri::LogicalSize::new(width.max(1.), height.max(1.)))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
fn read_data(
    app: tauri::AppHandle,
    webview: Webview,
    kind: String,
) -> Result<serde_json::Value, String> {
    trusted(&webview)?;
    let c = db(&app)?;
    match kind.as_str() {
        "settings" => {
            let v = c
                .query_row("SELECT value FROM settings WHERE key='state'", [], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap_or("{}".into());
            serde_json::from_str(&v).map_err(|e| e.to_string())
        }
        "history" | "bookmarks" => {
            let sql = if kind == "history" {
                "SELECT url,title FROM history ORDER BY visited DESC,id DESC LIMIT 500"
            } else {
                "SELECT url,title FROM bookmarks ORDER BY title"
            };
            let mut s = c.prepare(sql).map_err(|e| e.to_string())?;
            let rows = s
                .query_map([], |r| {
                    Ok(Entry {
                        url: r.get(0)?,
                        title: r.get(1)?,
                    })
                })
                .map_err(|e| e.to_string())?;
            let rows = rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            serde_json::to_value(rows).map_err(|e| e.to_string())
        }
        _ => Err("Données inconnues".into()),
    }
}
#[tauri::command]
fn write_data(
    app: tauri::AppHandle,
    webview: Webview,
    kind: String,
    value: String,
) -> Result<(), String> {
    trusted(&webview)?;
    let c = db(&app)?;
    match kind.as_str() {
        "settings" => { if value.len() > 1_000_000 { return Err("Paramètres trop volumineux".into()); } serde_json::from_str::<serde_json::Value>(&value).map_err(|e| e.to_string())?; c.execute("INSERT INTO settings VALUES('state',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [value]) },
        "bookmark" => { let e: Entry = serde_json::from_str(&value).map_err(|e| e.to_string())?; http_url(&e.url)?; c.execute("INSERT INTO bookmarks VALUES(?1,?2) ON CONFLICT(url) DO UPDATE SET title=excluded.title", params![e.url,e.title]) },
        "unbookmark" => c.execute("DELETE FROM bookmarks WHERE url=?1", [value]),
        "clear_history" => c.execute("DELETE FROM history", []),
        _ => return Err("Opération inconnue".into())
    }.map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
async fn clear_site_data(app: tauri::AppHandle, webview: Webview) -> Result<(), String> {
    trusted(&webview)?;
    for (label, view) in app.webviews() {
        if label.starts_with("tab-") {
            view.clear_all_browsing_data().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            #[cfg(debug_assertions)]
            if let Ok(base) = std::env::var("NOVA_SMOKE_URL") {
                smoke::run(_app.handle().clone(), base);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_tab,
            tab_action,
            layout,
            read_data,
            write_data,
            clear_site_data
        ])
        .run(tauri::generate_context!())
        .expect("Impossible de démarrer NOVA");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrations_preserve_data() {
        let c = Connection::open_in_memory().unwrap();
        migrate(&c).unwrap();
        c.execute(
            "INSERT INTO bookmarks VALUES(?1,?2)",
            params!["https://example.org", "Exemple"],
        )
        .unwrap();
        migrate(&c).unwrap();
        assert_eq!(
            c.query_row("SELECT title FROM bookmarks", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "Exemple"
        );
        c.pragma_update(None, "user_version", 2).unwrap();
        assert!(migrate(&c).is_err());
    }
    #[test]
    fn reject_unsafe_urls() {
        for u in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "https://user:pass@example.org",
            "data:text/html,hello",
        ] {
            assert!(http_url(u).is_err());
        }
    }
    #[test]
    fn accept_web_urls() {
        assert!(http_url("https://example.org/path?q=nova").is_ok());
        assert!(http_url("http://localhost:8000").is_ok());
    }
    #[test]
    fn isolate_shell() {
        assert!(tab_id("main").is_err());
        assert!(tab_id("tab-123").is_ok());
        assert!(tab_id("tab-../main").is_err());
    }
}

#[cfg(debug_assertions)]
mod smoke;
