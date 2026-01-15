use gtk4::prelude::*;
use gtk4::{gdk, gio};
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow};
use zbus::Connection;
use std::rc::Rc;
use std::cell::RefCell;
use gtk4::glib;

const APP_ID: &str = "org.carmenta.App";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let label = gtk4::Label::new(Some("Click to Insert/Copy Emoji"));
    let button = gtk4::Button::with_label("🙂 Smile");
    
    content.append(&label);
    content.append(&button);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Carmenta")
        .content(&content)
        .default_width(300)
        .default_height(200)
        .build();

    let window_weak = window.downgrade();
    
    button.connect_clicked(move |_| {
        let ctx = glib::MainContext::default();
        let win_weak_clone = window_weak.clone();

        ctx.spawn_local(async move {
            let text_to_insert = "🙂";
            match try_insert_via_extension(text_to_insert).await {
                Ok(_) => {
                    println!("Inserted via extension!");
                    if let Some(win) = win_weak_clone.upgrade() {
                         // win.close(); // Uncomment to close after insert
                    }
                },
                Err(e) => {
                    println!("Extension not found/error: {}. Fallback to Clipboard.", e);
                    copy_to_clipboard(text_to_insert);
                    if let Some(win) = win_weak_clone.upgrade() {
                        // win.close();
                    }
                }
            }
        });
    });

    window.present();
}

async fn try_insert_via_extension(text: &str) -> anyhow::Result<()> {
    let connection = Connection::session().await?;
    let proxy = connection.call_method(
        Some("org.gnome.Shell.Extensions.Carmenta"), // Bus Name (musi być taki sam jak własność extension)
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
