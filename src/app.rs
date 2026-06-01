use crate::config::AppConfig;
use crate::window::CarmentaWindow;
use clap::Parser;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::Application as GtkApplication;
use libadwaita::Application;
use std::cell::RefCell;
use std::rc::Rc;

// Global state to track insertion
thread_local! {
    pub static IS_INSERTING: RefCell<bool> = RefCell::new(false);
    static INSERT_TIMER: RefCell<Option<glib::SourceId>> = RefCell::new(None);
    static QUIT_REQUESTED: RefCell<bool> = RefCell::new(false);
    // The single, long-lived window. The app stays resident and merely hides
    // the window on dismiss, so re-invocation (via GApplication activate) just
    // re-shows an already-built, warm window instead of starting from scratch.
    static WINDOW: RefCell<Option<CarmentaWindow>> = RefCell::new(None);
    // Holds the app's use-count so it stays resident after the window hides.
    // Taken only by the primary instance on its first activation, so remote
    // (forwarding) invocations don't hold and exit normally.
    static HOLD: RefCell<Option<gtk4::gio::ApplicationHoldGuard>> = RefCell::new(None);
    static WINDOW_CONFIG: RefCell<Option<AppConfig>> = RefCell::new(None);
}

pub fn mark_inserting() {
    IS_INSERTING.with(|f| *f.borrow_mut() = true);

    INSERT_TIMER.with(|t| {
        if let Some(source) = t.borrow_mut().take() {
            source.remove();
        }
        let source = glib::timeout_add_local(std::time::Duration::from_millis(1000), || {
            IS_INSERTING.with(|f| *f.borrow_mut() = false);
            INSERT_TIMER.with(|t| *t.borrow_mut() = None);
            glib::ControlFlow::Break
        });
        *t.borrow_mut() = Some(source);
    });
}

/// Dismiss the picker: hide the window but keep the process resident so the
/// next invocation is instant. This is the normal "close" path (select, Esc,
/// focus loss, window close button).
pub fn hide_default() {
    WINDOW.with(|cell| {
        if let Some(win) = cell.borrow().as_ref() {
            win.hide();
        }
    });
}

/// Really quit the process (used by the menu's "Quit" action).
pub fn request_quit(app: &GtkApplication) {
    let should_quit = QUIT_REQUESTED.with(|flag| {
        let mut requested = flag.borrow_mut();
        if *requested {
            false
        } else {
            *requested = true;
            true
        }
    });

    if !should_quit {
        return;
    }

    let app_weak = app.downgrade();
    glib::idle_add_local_once(move || {
        if let Some(app) = app_weak.upgrade() {
            for window in app.windows() {
                window.close();
            }
            app.quit();
        }
    });
}

pub struct CarmentaApp {
    app: Application,
}

impl CarmentaApp {
    pub fn new(app_id: &str, config: AppConfig) -> Self {
        let app = Application::builder()
            .application_id(app_id)
            .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();

        let config = Rc::new(RefCell::new(config));

        app.connect_activate({
            let config = config.clone();
            move |app| Self::on_activate(app, &config.borrow())
        });

        app.connect_command_line(move |app, command_line| {
            let args = command_line.arguments();
            match AppConfig::try_parse_from(args) {
                Ok(parsed) => {
                    *config.borrow_mut() = parsed.clone();
                    Self::on_activate(app, &parsed);
                    0
                }
                Err(err) => {
                    eprint!("{err}");
                    err.exit_code()
                }
            }
        });

        Self { app }
    }

    pub fn run(&self) {
        let args = std::env::args().collect::<Vec<_>>();
        self.app.run_with_args(&args);
    }

    fn on_activate(app: &Application, config: &AppConfig) {
        crate::set_close_on_select(config.close_on_select);

        // prefetching DBus connection to avoid flicker on first insert
        crate::dbus::DBusClient::init_connection();

        WINDOW.with(|cell| {
            let mut cell = cell.borrow_mut();
            let should_rebuild = WINDOW_CONFIG.with(|stored| {
                stored
                    .borrow()
                    .as_ref()
                    .map(|old| old.gifs_enabled() != config.gifs_enabled())
                    .unwrap_or(false)
            });

            if should_rebuild {
                if let Some(win) = cell.take() {
                    win.destroy();
                }
            }

            if let Some(win) = cell.as_ref() {
                // Resident instance re-invoked: just re-show the warm window.
                win.apply_config(config);
                if config.prewarm {
                    win.prewarm();
                } else {
                    win.show();
                }
            } else {
                // First activation in the primary instance: become resident by
                // holding the app so hiding the window won't quit the process.
                HOLD.with(|h| *h.borrow_mut() = Some(app.hold()));
                let win = CarmentaWindow::new(app, config);
                if config.prewarm {
                    // Background autostart: warm resources, stay hidden.
                    win.prewarm();
                } else {
                    win.show();
                }
                *cell = Some(win);
            }
        });

        WINDOW_CONFIG.with(|stored| *stored.borrow_mut() = Some(config.clone()));
    }
}
