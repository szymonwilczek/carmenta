mod app;
mod config;
mod dbus;
mod history;
mod ui;
mod window;

#[allow(unused_imports)]
use app::CarmentaApp;
use clap::Parser;
use config::AppConfig;
use std::sync::OnceLock;

const APP_ID: &str = "io.github.szymonwilczek.carmenta";

pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
pub static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
pub static CLOSE_ON_SELECT: OnceLock<bool> = OnceLock::new();

/// Whether the window should close automatically after an item is selected.
pub fn close_on_select() -> bool {
    CLOSE_ON_SELECT.get().copied().unwrap_or(false)
}

fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    CLOSE_ON_SELECT.set(config.close_on_select).ok();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    RUNTIME.set(rt).expect("Failed to set global runtime");
    
    let client = reqwest::Client::new();
    CLIENT.set(client).expect("Failed to set global client");

    let app = CarmentaApp::new(APP_ID, config);
    app.run();

    Ok(())
}
