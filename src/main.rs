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

fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    RUNTIME.set(rt).expect("Failed to set global runtime");
    
    let client = reqwest::Client::new();
    CLIENT.set(client).expect("Failed to set global client");

    let app = CarmentaApp::new(APP_ID, config);
    app.run();

    Ok(())
}
