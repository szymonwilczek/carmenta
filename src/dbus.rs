use zbus::Connection;
use gtk4::gdk;
use gtk4::prelude::*;
use tokio::sync::OnceCell;

pub struct DBusClient;

// global async-safe connection cache
static CONNECTION: OnceCell<Connection> = OnceCell::const_new();

impl DBusClient {
    pub fn insert_or_copy(text: &str) {
        // Fire and forget on the runtime
        let text_owned = text.to_string();
        if let Some(rt) = crate::RUNTIME.get() {
            rt.spawn(async move {
                 match Self::try_insert_via_extension(&text_owned).await {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("DBus error: {}", e);
                        // fallback: copy to clipboard and quit app (no extension mode)
                        let text_clone = text_owned.clone();
                        gtk4::glib::MainContext::default().invoke(move || {
                            Self::copy_to_clipboard(&text_clone);
                            // quit the app after copying (non-extension behavior)
                            gtk4::glib::timeout_add_local_once(
                                std::time::Duration::from_millis(100),
                                || {
                                    if let Some(app) = gtk4::gio::Application::default() {
                                        app.quit();
                                    }
                                }
                            );
                        });
                    }
                 }
            });
        } else {
            eprintln!("Runtime not initialized!");
        }
    }

    pub fn pin_window(pinned: bool) {
        if let Some(rt) = crate::RUNTIME.get() {
            rt.spawn(async move {
                match Self::get_connection().await {
                    Ok(conn) => {
                        let _ = conn.call_method(
                            Some("org.gnome.Shell.Extensions.Carmenta"), 
                            "/org/gnome/Shell/Extensions/Carmenta",    
                            Some("org.gnome.Shell.Extensions.Carmenta"), 
                            "PinWindow",
                            &(pinned),
                        ).await;
                    }
                    Err(e) => eprintln!("DBus connection error: {}", e),
                }
            });
        }
    }

    async fn get_connection() -> anyhow::Result<Connection> {
        // get or initialize connection
        let conn: &Connection = CONNECTION.get_or_try_init(|| async {
            Connection::session().await
        }).await?;
        
        Ok(conn.clone())
    }

    async fn try_insert_via_extension(text: &str) -> anyhow::Result<()> {
        let connection = Self::get_connection().await?;
        
        let _reply = connection.call_method(
            Some("org.gnome.Shell.Extensions.Carmenta"), 
            "/org/gnome/Shell/Extensions/Carmenta",    
            Some("org.gnome.Shell.Extensions.Carmenta"), 
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
