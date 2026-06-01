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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

const APP_ID: &str = "io.github.szymonwilczek.carmenta";

pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
pub static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
pub static CLOSE_ON_SELECT: AtomicBool = AtomicBool::new(false);

/// Whether the window should close automatically after an item is selected.
pub fn close_on_select() -> bool {
    CLOSE_ON_SELECT.load(Ordering::Relaxed)
}

pub fn set_close_on_select(value: bool) {
    CLOSE_ON_SELECT.store(value, Ordering::Relaxed);
}

/// The HTTP client, built lazily on first network use. Keeping the (TLS-heavy)
/// client off the startup path keeps emoji/kaomoji/symbol use network-free.
pub fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new)
}

fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    set_close_on_select(config.close_on_select);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    RUNTIME.set(rt).expect("Failed to set global runtime");

    let app = CarmentaApp::new(APP_ID, config);
    app.run();

    Ok(())
}
