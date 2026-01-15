use gtk4::prelude::*;
use libadwaita::prelude::ApplicationExt;
use libadwaita::Application;
use crate::window::CarmentaWindow;

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
        let window = CarmentaWindow::new(app);
        window.present();
    }
}
