use gtk4::prelude::*;
use libadwaita::Application;
use std::cell::RefCell;
use crate::window::CarmentaWindow;

// Global state to track if we are in the middle of an insertion (and thus expecting focus loss)
thread_local! {
    pub static IS_INSERTING: RefCell<bool> = RefCell::new(false);
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
