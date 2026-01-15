mod app;
mod window;
mod dbus;
mod ui;

use app::CarmentaApp;

const APP_ID: &str = "org.carmenta.App";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = CarmentaApp::new(APP_ID);
    app.run();
    
    Ok(())
}
