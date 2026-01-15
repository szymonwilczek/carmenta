mod app;
mod window;
mod dbus;
mod ui;

#[allow(unused_imports)]
use app::CarmentaApp;
use std::sync::OnceLock;

const APP_ID: &str = "org.carmenta.App";

pub static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    RUNTIME.set(rt).expect("Failed to set global runtime");

    let app = CarmentaApp::new(APP_ID);
    app.run();
    
    Ok(())
}
