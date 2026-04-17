use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::models::webapp::WebApp;
use crate::paths::ManagedPaths;

const SCHEMA_VERSION: i64 = 2;

pub fn init(paths: &ManagedPaths) -> Result<()> {
    let connection = Connection::open(&paths.db_path)
        .with_context(|| format!("Failed to open database {}.", paths.db_path.display()))?;

    migrate(&connection)?;

    Ok(())
}

fn migrate(connection: &Connection) -> Result<()> {
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if version < 1 || !table_exists(connection, "webapps")? {
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS webapps (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                url         TEXT NOT NULL UNIQUE,
                icon_path   TEXT,
                category    TEXT NOT NULL DEFAULT 'Internet',
                browser     TEXT NOT NULL DEFAULT 'auto',
                nav_bar     INTEGER NOT NULL DEFAULT 0,
                isolated    INTEGER NOT NULL DEFAULT 1,
                tray        INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        )?;
    }

    if version < 2 || !column_exists(connection, "webapps", "window_mode")? {
        ensure_column(
            connection,
            "window_mode",
            "ALTER TABLE webapps ADD COLUMN window_mode TEXT NOT NULL DEFAULT 'normal'",
        )?;
    }

    connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

pub fn list_webapps(paths: &ManagedPaths) -> Result<Vec<WebApp>> {
    let connection = open(paths)?;
    let mut statement = connection.prepare(
        r#"
        SELECT id, name, url, icon_path, category, browser, nav_bar, isolated, tray, window_mode, created_at, updated_at
        FROM webapps
        ORDER BY lower(name) ASC
        "#,
    )?;

    let rows = statement.query_map([], map_webapp_row)?;
    let mut items = Vec::new();

    for row in rows {
        items.push(row?);
    }

    Ok(items)
}

pub fn find_webapp(paths: &ManagedPaths, target: &str) -> Result<Option<WebApp>> {
    let connection = open(paths)?;
    let mut statement = connection.prepare(
        r#"
        SELECT id, name, url, icon_path, category, browser, nav_bar, isolated, tray, window_mode, created_at, updated_at
        FROM webapps
        WHERE id = ?1 OR lower(name) = lower(?1)
        LIMIT 1
        "#,
    )?;

    let item = statement.query_row([target], map_webapp_row).optional()?;

    Ok(item)
}

pub fn insert_webapp(paths: &ManagedPaths, webapp: &WebApp) -> Result<()> {
    let connection = open(paths)?;
    connection.execute(
        r#"
        INSERT INTO webapps (id, name, url, icon_path, category, browser, nav_bar, isolated, tray, window_mode)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            webapp.id,
            webapp.name,
            webapp.url,
            webapp.icon_path,
            webapp.category,
            webapp.browser.as_str(),
            webapp.nav_bar as i64,
            webapp.isolated as i64,
            webapp.tray as i64,
            webapp.window_mode.as_str()
        ],
    )?;

    Ok(())
}

pub fn update_webapp(paths: &ManagedPaths, webapp: &WebApp) -> Result<()> {
    let connection = open(paths)?;
    connection.execute(
        r#"
        UPDATE webapps
        SET name = ?2,
            url = ?3,
            icon_path = ?4,
            category = ?5,
            browser = ?6,
            nav_bar = ?7,
            isolated = ?8,
            tray = ?9,
            window_mode = ?10,
            updated_at = datetime('now')
        WHERE id = ?1
        "#,
        params![
            webapp.id,
            webapp.name,
            webapp.url,
            webapp.icon_path,
            webapp.category,
            webapp.browser.as_str(),
            webapp.nav_bar as i64,
            webapp.isolated as i64,
            webapp.tray as i64,
            webapp.window_mode.as_str()
        ],
    )?;

    Ok(())
}

pub fn delete_webapp(paths: &ManagedPaths, id: &str) -> Result<()> {
    let connection = open(paths)?;
    connection.execute("DELETE FROM webapps WHERE id = ?1", [id])?;
    Ok(())
}

pub fn ensure_unique_url(paths: &ManagedPaths, url: &str, exclude_id: Option<&str>) -> Result<()> {
    let connection = open(paths)?;
    let existing: Option<String> = connection
        .query_row(
            "SELECT id FROM webapps WHERE url = ?1 LIMIT 1",
            [url],
            |row| row.get(0),
        )
        .optional()?;

    match (existing.as_deref(), exclude_id) {
        (Some(existing_id), Some(ignore_id)) if existing_id == ignore_id => Ok(()),
        (Some(_), _) => Err(anyhow!("A web app with this URL already exists.")),
        (None, _) => Ok(()),
    }
}

fn open(paths: &ManagedPaths) -> Result<Connection> {
    Connection::open(&paths.db_path)
        .with_context(|| format!("Failed to open database {}.", paths.db_path.display()))
}

fn table_exists(connection: &Connection, table_name: &str) -> Result<bool> {
    let exists = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1",
            [table_name],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    Ok(exists)
}

fn column_exists(connection: &Connection, table_name: &str, column_name: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({table_name})");
    let mut query = connection.prepare(&pragma)?;
    let columns = query.query_map([], |row| row.get::<_, String>(1))?;

    for column in columns {
        if column? == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}

fn ensure_column(connection: &Connection, column_name: &str, statement: &str) -> Result<()> {
    if column_exists(connection, "webapps", column_name)? {
        return Ok(());
    }

    connection.execute(statement, [])?;
    Ok(())
}

fn map_webapp_row(row: &Row<'_>) -> rusqlite::Result<WebApp> {
    Ok(WebApp {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        icon_path: row.get(3)?,
        icon_data_url: None,
        category: row.get(4)?,
        browser: crate::models::webapp::BrowserChoice::from_db(&row.get::<_, String>(5)?),
        nav_bar: row.get::<_, i64>(6)? != 0,
        isolated: row.get::<_, i64>(7)? != 0,
        tray: row.get::<_, i64>(8)? != 0,
        window_mode: crate::models::webapp::WindowMode::from_db(&row.get::<_, String>(9)?),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::models::webapp::{BrowserChoice, WindowMode};

    fn temp_paths() -> ManagedPaths {
        let root = std::env::temp_dir().join(format!("webapp-manager-db-{}", uuid::Uuid::new_v4()));
        let data_dir = root.join("data");
        let config_dir = root.join("config");
        let state_dir = root.join("state");
        let icons_dir = data_dir.join("icons");
        let profiles_dir = data_dir.join("profiles");
        let desktop_dir = data_dir.join("applications");

        for directory in [
            &data_dir,
            &config_dir,
            &state_dir,
            &icons_dir,
            &profiles_dir,
            &desktop_dir,
        ] {
            fs::create_dir_all(directory).unwrap();
        }

        ManagedPaths {
            data_dir: data_dir.clone(),
            config_dir,
            state_dir,
            db_path: data_dir.join("webapps.db"),
            icons_dir,
            profiles_dir,
            desktop_dir,
        }
    }

    fn sample_webapp(id: &str, name: &str, url: &str) -> WebApp {
        WebApp {
            id: id.to_owned(),
            name: name.to_owned(),
            url: url.to_owned(),
            icon_path: Some(format!("/tmp/{id}.png")),
            icon_data_url: None,
            category: "Internet".to_owned(),
            browser: BrowserChoice::Chrome,
            nav_bar: true,
            isolated: true,
            tray: false,
            window_mode: WindowMode::Normal,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn crud_round_trip_persists_and_updates_webapps() {
        let paths = temp_paths();
        init(&paths).unwrap();

        let mut webapp = sample_webapp("mail", "Mail", "https://mail.example.com");
        insert_webapp(&paths, &webapp).unwrap();

        let stored = find_webapp(&paths, "mail").unwrap().unwrap();
        assert_eq!(stored.name, "Mail");
        assert_eq!(stored.url, "https://mail.example.com");
        assert_eq!(stored.browser, BrowserChoice::Chrome);
        assert!(stored.nav_bar);
        assert!(stored.isolated);
        assert_eq!(stored.window_mode, WindowMode::Normal);

        webapp.name = "Personal Mail".to_owned();
        webapp.url = "https://mail.example.com/app".to_owned();
        webapp.browser = BrowserChoice::Firefox;
        webapp.nav_bar = false;
        webapp.isolated = false;
        webapp.tray = true;
        webapp.window_mode = WindowMode::Fullscreen;
        update_webapp(&paths, &webapp).unwrap();

        let updated = find_webapp(&paths, "personal mail").unwrap().unwrap();
        assert_eq!(updated.name, "Personal Mail");
        assert_eq!(updated.url, "https://mail.example.com/app");
        assert_eq!(updated.browser, BrowserChoice::Firefox);
        assert!(!updated.nav_bar);
        assert!(!updated.isolated);
        assert!(updated.tray);
        assert_eq!(updated.window_mode, WindowMode::Fullscreen);

        delete_webapp(&paths, "mail").unwrap();
        assert!(find_webapp(&paths, "mail").unwrap().is_none());

        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn init_sets_schema_version() {
        let paths = temp_paths();

        init(&paths).unwrap();

        let connection = open(&paths).unwrap();
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn list_webapps_is_sorted_case_insensitively() {
        let paths = temp_paths();
        init(&paths).unwrap();

        insert_webapp(
            &paths,
            &sample_webapp("zeta", "zeta", "https://zeta.example.com"),
        )
        .unwrap();
        insert_webapp(
            &paths,
            &sample_webapp("alpha", "Alpha", "https://alpha.example.com"),
        )
        .unwrap();
        insert_webapp(
            &paths,
            &sample_webapp("beta", "beta", "https://beta.example.com"),
        )
        .unwrap();

        let names: Vec<String> = list_webapps(&paths)
            .unwrap()
            .into_iter()
            .map(|item| item.name)
            .collect();

        assert_eq!(names, vec!["Alpha", "beta", "zeta"]);
        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn ensure_unique_url_rejects_duplicates_but_allows_same_id() {
        let paths = temp_paths();
        init(&paths).unwrap();
        insert_webapp(
            &paths,
            &sample_webapp("mail", "Mail", "https://mail.example.com"),
        )
        .unwrap();

        let duplicate = ensure_unique_url(&paths, "https://mail.example.com", None);
        assert!(duplicate.is_err());

        let same_id = ensure_unique_url(&paths, "https://mail.example.com", Some("mail"));
        assert!(same_id.is_ok());

        let unique = ensure_unique_url(&paths, "https://calendar.example.com", None);
        assert!(unique.is_ok());

        fs::remove_dir_all(paths.data_dir.parent().unwrap()).unwrap();
    }
}
