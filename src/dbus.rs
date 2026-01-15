use zbus::Connection;
use gtk4::gdk;
use gtk4::prelude::*;

pub struct DBusClient;

impl DBusClient {
    pub async fn insert_or_copy(text: &str) {
        match Self::try_insert_via_extension(text).await {
            Ok(_) => println!("Inserted via extension: {}", text),
            Err(e) => {
                println!("Extension error: {}. Fallback to Clipboard.", e);
                Self::copy_to_clipboard(text);
            }
        }
    }

    async fn try_insert_via_extension(text: &str) -> anyhow::Result<()> {
        let connection = Connection::session().await?;
        let _reply = connection.call_method(
            Some("org.gnome.Shell.Extensions.Carmenta"), // Bus Name
            "/org/gnome/Shell/Extensions/Carmenta",    // Object Path
            Some("org.gnome.Shell.Extensions.Carmenta"), // Interface
            "InsertText",
            &(text),
        ).await?;
        
        Ok(())
    }

    fn copy_to_clipboard(text: &str) {
        let display = gdk::Display::default().expect("No display");
        let clipboard = display.clipboard();
        clipboard.set_text(text);
        println!("Copied to clipboard: {}", text);
    }
}
