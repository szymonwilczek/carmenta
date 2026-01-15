use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{Application, ApplicationWindow};
use gtk4::{Box, Orientation, SearchEntry};
use gtk4::glib;

pub struct CarmentaWindow {
    window: ApplicationWindow,
}

impl CarmentaWindow {
    pub fn new(app: &Application) -> Self {
        let content = Box::new(Orientation::Vertical, 0);

        // 1. Search Bar
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search Emoji, Kaomoji, Symbols...")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        content.append(&search_entry);

        // 2. View Stack (Tabs)
        let stack = libadwaita::ViewStack::new();
        
        // -- Emoji Page --
        let emoji_page = crate::ui::emoji_grid::create_emoji_grid(&search_entry);
        // emoji_page is now a gtk::Box, which implements IsA<Widget>, so this is fine.
        stack.add_titled(&emoji_page, Some("emoji"), "Emoji");

        // -- Kaomoji Page --
        let kaomoji_page = gtk4::Label::new(Some("(╯°□°)╯︵ ┻━┻"));
        stack.add_titled(&kaomoji_page, Some("kaomoji"), "Kaomoji");

        // -- Clipboard Page --
        let clipboard_page = gtk4::Label::new(Some("Clipboard History"));
        stack.add_titled(&clipboard_page, Some("clipboard"), "Clipboard");

        // View Switcher (Bottom Bar)
        let view_switcher = libadwaita::ViewSwitcherBar::builder()
            .stack(&stack)
            .reveal(true)
            .build();

        // Assemble Window Content
        let main_box = Box::new(Orientation::Vertical, 0);
        main_box.append(&content); // Search
        let expanded_stack = stack.clone();
        expanded_stack.set_vexpand(true);
        main_box.append(&expanded_stack); // Content
        main_box.append(&view_switcher); // Tabs

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Carmenta")
            .content(&main_box)
            .default_width(420)
            .default_height(480)
            .modal(false) // Non-modal to interact with other apps
            .decorated(true) 
            .build();
            
        // Pin window to stay on top
        crate::dbus::DBusClient::pin_window(true);

        let window_clone = window.clone();
        window.connect_is_active_notify(move |win| {
            if !win.is_active() {
                // Focus lost!
                let is_inserting = crate::app::IS_INSERTING.with(|f| *f.borrow());
                if !is_inserting {
                    println!("Focus lost and not inserting -> Closing App");
                    if let Some(app) = win.application() {
                        app.quit();
                    }
                } else {
                    println!("Focus lost but inserting -> Keeping Open");
                }
            }
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
