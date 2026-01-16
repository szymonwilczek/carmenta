use zbus::Connection;
use gtk4::gdk;
use gtk4::prelude::*;
use tokio::sync::OnceCell;
use std::time::Duration;

pub struct DBusClient;

// global async-safe connection cache
static CONNECTION: OnceCell<Connection> = OnceCell::const_new();

// timeout for DBus operations
const DBUS_TIMEOUT: Duration = Duration::from_millis(500);

impl DBusClient {
    pub fn insert_or_copy(text: &str) {
        let text_owned = text.to_string();
        if let Some(rt) = crate::RUNTIME.get() {
            rt.spawn(async move {
                // wrap the extension call with a timeout
                let result = tokio::time::timeout(
                    DBUS_TIMEOUT,
                    Self::try_insert_via_extension(&text_owned)
                ).await;
                
                match result {
                    Ok(Ok(_)) => {}, // success
                    Ok(Err(e)) => {
                        eprintln!("DBus error: {}", e);
                        Self::fallback_copy_and_quit(text_owned);
                    }
                    Err(_) => {
                        eprintln!("DBus timeout: extension did not respond in {:?}", DBUS_TIMEOUT);
                        Self::fallback_copy_and_quit(text_owned);
                    }
                }
            });
        } else {
            eprintln!("Runtime not initialized!");
        }
    }
    
    fn fallback_copy_and_quit(text: String) {
        gtk4::glib::MainContext::default().invoke(move || {
            Self::copy_to_clipboard(&text);
            gtk4::glib::timeout_add_local_once(
                Duration::from_millis(100),
                || {
                    if let Some(app) = gtk4::gio::Application::default() {
                        app.quit();
                    }
                }
            );
        });
    }

    pub fn pin_window(pinned: bool) {
        if let Some(rt) = crate::RUNTIME.get() {
            rt.spawn(async move {
                // shorter timeout for pin_window as its non-critical
                let result = tokio::time::timeout(
                    Duration::from_millis(200),
                    Self::do_pin_window(pinned)
                ).await;
                
                if let Err(_) = result {
                    eprintln!("DBus timeout: pin_window did not complete");
                }
            });
        }
    }
    
    async fn do_pin_window(pinned: bool) -> anyhow::Result<()> {
        let conn = Self::get_connection().await?;
        conn.call_method(
            Some("org.gnome.Shell.Extensions.Carmenta"), 
            "/org/gnome/Shell/Extensions/Carmenta",    
            Some("org.gnome.Shell.Extensions.Carmenta"), 
            "PinWindow",
            &(pinned),
        ).await?;
        Ok(())
    }

    async fn get_connection() -> anyhow::Result<Connection> {
        let conn: &Connection = CONNECTION.get_or_try_init(|| async {
            Connection::session().await
        }).await?;
        
        Ok(conn.clone())
    }

    async fn try_insert_via_extension(text: &str) -> anyhow::Result<()> {
        let connection = Self::get_connection().await?;
        
        connection.call_method(
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
