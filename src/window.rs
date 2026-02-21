use crate::config::AppConfig;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{gio, Box, Orientation, SearchEntry};
use libadwaita::{Application, ApplicationWindow};
use std::cell::RefCell;
use std::rc::Rc;

pub struct CarmentaWindow {
    pub window: ApplicationWindow,
}

impl CarmentaWindow {
    pub fn new(app: &Application, config: &AppConfig) -> Self {
        // Menu
        let menu = gio::Menu::new();
        menu.append(Some("About Carmenta"), Some("app.about"));
        menu.append(Some("Quit"), Some("app.quit"));

        // Actions (App Scope)
        if !app.has_action("about") {
            let action_about = gio::SimpleAction::new("about", None);
            action_about.connect_activate(|_, _| {
                let _ = gio::AppInfo::launch_default_for_uri(
                    "https://github.com/szymonwilczek/carmenta",
                    None::<&gio::AppLaunchContext>,
                );
            });
            app.add_action(&action_about);
        }

        if !app.has_action("quit") {
            let action_quit = gio::SimpleAction::new("quit", None);
            let app_weak = app.downgrade();
            action_quit.connect_activate(move |_, _| {
                if let Some(a) = app_weak.upgrade() {
                    crate::app::request_quit(a.upcast_ref());
                }
            });
            app.add_action(&action_quit);
        }

        // Top Bar Layout (Search + Menu)
        let top_bar = Box::new(Orientation::Horizontal, 6);
        top_bar.set_margin_top(12);
        top_bar.set_margin_bottom(12);
        top_bar.set_margin_start(12);
        top_bar.set_margin_end(12);

        // Search Bar
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search...")
            .hexpand(true) // available width
            .build();

        // Menu Button
        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&menu)
            .valign(gtk4::Align::Center)
            .build();

        top_bar.append(&search_entry);
        top_bar.append(&menu_button);

        // Main Layout
        let content = Box::new(Orientation::Vertical, 0);
        content.append(&top_bar);

        let stack = libadwaita::ViewStack::new();

        let emoji_page = crate::ui::emoji_grid::create_emoji_grid(&search_entry);
        let page = stack.add_titled(&emoji_page, Some("emoji"), "Emoji");
        page.set_icon_name(Some("face-smile-symbolic"));

        let kaomoji_page = crate::ui::kaomoji_grid::create_kaomoji_grid(&search_entry);
        let page = stack.add_titled(&kaomoji_page, Some("kaomoji"), "Kaomoji");
        page.set_icon_name(Some("face-wink-symbolic"));

        let symbols_page = crate::ui::symbols_grid::create_symbols_grid(&search_entry);
        let page = stack.add_titled(&symbols_page, Some("symbols"), "Symbols");
        page.set_icon_name(Some("preferences-desktop-font-symbolic"));

        if config.gifs_enabled() {
            let gif_page = crate::ui::gif_grid::create_gif_grid(&search_entry);
            let page = stack.add_titled(&gif_page, Some("gifs"), "GIFs");
            page.set_icon_name(Some("emblem-photos-symbolic"));
        }

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
            .default_width(config.width)
            .default_height(config.height)
            .modal(false) // non-modal to interact with other apps
            .decorated(true)
            .build();

        // pin window to stay on top - but wait for window to be mapped!
        let win_weak_pin = window.downgrade();
        window.connect_map(move |_| {
            if let Some(_) = win_weak_pin.upgrade() {
                crate::dbus::DBusClient::pin_window(true);
            }
        });

        window.connect_close_request(move |win| {
            crate::dbus::DBusClient::pin_window(false);
            if let Some(app) = win.application() {
                crate::app::request_quit(&app);
            }
            glib::Propagation::Proceed
        });

        let focus_loss_checker: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        window.connect_is_active_notify(glib::clone!(
            #[strong]
            focus_loss_checker,
            move |win| {
                if win.is_active() {
                    if let Some(source_id) = focus_loss_checker.borrow_mut().take() {
                        source_id.remove();
                    }
                    return;
                }

                if focus_loss_checker.borrow().is_some() {
                    return;
                }

                let win_weak = win.downgrade();
                let focus_loss_checker_for_timer = focus_loss_checker.clone();
                let checker =
                    glib::timeout_add_local(std::time::Duration::from_millis(120), move || {
                        let Some(w) = win_weak.upgrade() else {
                            *focus_loss_checker_for_timer.borrow_mut() = None;
                            return glib::ControlFlow::Break;
                        };

                        if w.is_active() {
                            *focus_loss_checker_for_timer.borrow_mut() = None;
                            return glib::ControlFlow::Break;
                        }

                        let is_inserting = crate::app::IS_INSERTING.with(|f| *f.borrow());
                        if is_inserting {
                            return glib::ControlFlow::Continue;
                        }

                        println!("Focus lost confirmed -> Closing App");
                        if let Some(app) = w.application() {
                            crate::app::request_quit(&app);
                        }
                        *focus_loss_checker_for_timer.borrow_mut() = None;
                        glib::ControlFlow::Break
                    });

                *focus_loss_checker.borrow_mut() = Some(checker);
            }
        ));

        // Escape Key handler
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let app_weak_key = app.downgrade();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                if let Some(a) = app_weak_key.upgrade() {
                    crate::app::request_quit(a.upcast_ref());
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
