mod app;
mod config;
mod ui;
mod services;

use gtk4::{Application, prelude::*};
use app::BarApp;

fn main() {
    logger::init();
    
    let app = Application::builder()
        .application_id("rs.regimentor.oxidbar")
        .build();

    let bar_app = BarApp::new();
    app.connect_activate(move |app| {
        bar_app.build_ui(app);
    });
    
    app.run();
}
