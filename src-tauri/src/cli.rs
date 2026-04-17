use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use crate::commands::webapp;
use crate::db;
use crate::models::webapp::{BrowserChoice, WebAppPayload, WindowMode};
use crate::paths::ManagedPaths;

#[derive(Debug, Parser)]
#[command(name = "linux-pwa-manager")]
#[command(about = "Turn websites into Linux desktop applications.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Launch {
        target: String,
    },
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        url: String,
        #[arg(long, default_value = "Internet")]
        category: String,
        #[arg(long, value_enum)]
        browser: Option<BrowserChoice>,
        #[arg(long, default_value_t = false)]
        nav_bar: bool,
        #[arg(long, default_value_t = true)]
        isolated: bool,
        #[arg(long, default_value_t = false)]
        tray: bool,
        #[arg(long, value_enum)]
        window_mode: Option<WindowMode>,
    },
    Remove {
        target: String,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Edit {
        target: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long, value_enum)]
        browser: Option<BrowserChoice>,
        #[arg(long)]
        nav_bar: Option<bool>,
        #[arg(long)]
        isolated: Option<bool>,
        #[arg(long)]
        tray: Option<bool>,
        #[arg(long, value_enum)]
        window_mode: Option<WindowMode>,
    },
    Browsers,
    Export {
        path: String,
    },
    Import {
        path: String,
        #[arg(long, default_value_t = false)]
        replace_existing: bool,
    },
}

pub async fn run(command: Command) -> Result<()> {
    let paths = ManagedPaths::discover()?;
    db::init(&paths)?;

    match command {
        Command::Launch { target } => webapp::launch(None, &paths, &target)?,
        Command::Add {
            name,
            url,
            category,
            browser,
            nav_bar,
            isolated,
            tray,
            window_mode,
        } => {
            let settings = crate::commands::settings::load_settings(&paths);
            let timeout_secs = settings.http_timeout_secs;
            let max_bytes = settings.max_icon_size_mb * 1024 * 1024;
            let default_browser = settings.default_browser.clone();
            let default_window_mode = settings.default_window_mode.clone();
            let item = webapp::create(
                &paths,
                WebAppPayload {
                    id: None,
                    name,
                    url,
                    category,
                    browser: browser.unwrap_or(default_browser),
                    nav_bar,
                    isolated,
                    tray,
                    window_mode: window_mode.unwrap_or(default_window_mode),
                    uploaded_icon_data_url: None,
                    icon_source_url: None,
                    clear_icon: false,
                },
                timeout_secs,
                max_bytes,
            )
            .await?;
            println!("Created {} ({})", item.name, item.id);
        }
        Command::Remove { target } => {
            webapp::delete(&paths, &target)?;
            println!("Removed {target}");
        }
        Command::List { json } => {
            let items = webapp::list(&paths)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&items)?);
            } else {
                for item in items {
                    println!("{}\t{}\t{}", item.id, item.name, item.url);
                }
            }
        }
        Command::Edit {
            target,
            name,
            url,
            category,
            browser,
            nav_bar,
            isolated,
            tray,
            window_mode,
        } => {
            let existing = crate::db::find_webapp(&paths, &target)?
                .ok_or_else(|| anyhow!("Web app not found."))?;
            let settings = crate::commands::settings::load_settings(&paths);
            let timeout_secs = settings.http_timeout_secs;
            let max_bytes = settings.max_icon_size_mb * 1024 * 1024;
            let updated = webapp::update(
                &paths,
                WebAppPayload {
                    id: Some(existing.id),
                    name: name.unwrap_or(existing.name),
                    url: url.unwrap_or(existing.url),
                    category: category.unwrap_or(existing.category),
                    browser: browser.unwrap_or(existing.browser),
                    nav_bar: nav_bar.unwrap_or(existing.nav_bar),
                    isolated: isolated.unwrap_or(existing.isolated),
                    tray: tray.unwrap_or(existing.tray),
                    window_mode: window_mode.unwrap_or(existing.window_mode),
                    uploaded_icon_data_url: None,
                    icon_source_url: None,
                    clear_icon: false,
                },
                timeout_secs,
                max_bytes,
            )
            .await?;
            println!("Updated {} ({})", updated.name, updated.id);
        }
        Command::Browsers => {
            for browser in crate::runtime::browser::detect_browsers() {
                println!(
                    "{}\t{}\t{}",
                    browser.id.as_str(),
                    browser.name,
                    browser.version.unwrap_or_else(|| "unknown".to_owned())
                );
            }
        }
        Command::Export { path } => {
            crate::commands::backup::export_to_path(&paths, std::path::Path::new(&path))?;
            println!("Exported backup to {path}");
        }
        Command::Import {
            path,
            replace_existing,
        } => {
            crate::commands::backup::import_from_path(
                &paths,
                std::path::Path::new(&path),
                replace_existing,
            )?;
            println!("Imported backup from {path}");
        }
    }

    Ok(())
}
