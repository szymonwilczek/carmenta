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
        let emoji_page = crate::ui::emoji_grid::create_emoji_grid();
        // emoji_page is now a gtk::Box, which implements IsA<Widget>, so this is fine.
        stack.add_titled(&emoji_page, Some("emoji"), "Emoji");

        /* 
        // Old Dummy Logic
        let emoji_page = Box::new(Orientation::Vertical, 12);
        // ...
        */

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
            .default_width(400) // Węższe, bardziej jak "popover"
            .default_height(500)
            .build();

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
