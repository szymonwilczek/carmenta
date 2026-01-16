use gtk4::prelude::*;
use gtk4::glib;
use libadwaita::Application;
use std::cell::RefCell;
use crate::window::CarmentaWindow;

// Global state to track insertion
thread_local! {
    pub static IS_INSERTING: RefCell<bool> = RefCell::new(false);
    static INSERT_TIMER: RefCell<Option<glib::SourceId>> = RefCell::new(None);
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

pub struct CarmentaApp {
    app: Application,
}

impl CarmentaApp {
    pub fn new(app_id: &str) -> Self {
        let app = Application::builder()
            .application_id(app_id)
            .build();

        app.connect_activate(Self::on_activate);

        Self { app }
    }

    pub fn run(&self) {
        self.app.run();
    }

    fn on_activate(app: &Application) {
        // prefetching DBus connection to avoid flicker on first insert
        crate::dbus::DBusClient::init_connection();
        
        let window = CarmentaWindow::new(app);
        window.present();
    }
}
