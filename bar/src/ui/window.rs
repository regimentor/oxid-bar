use gtk4::{Align, ApplicationWindow, gdk::Display, gdk::Monitor, prelude::*};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::config::BarConfig;

/// Настраивает окно приложения для Wayland layer shell
pub fn setup_window(window: &ApplicationWindow, config: &BarConfig) {
    window.set_hexpand(true);
    window.set_halign(Align::Fill);
    window.set_default_height(config.height);

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace(Some("oxidbar"));
    window.auto_exclusive_zone_enable();
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 0);
    window.set_margin(Edge::Left, 0);
    window.set_margin(Edge::Right, 0);
    
    set_full_width(window);
    window.set_decorated(false);
    window.set_resizable(false);
}

fn set_full_width(window: &ApplicationWindow) {
    if let Some(display) = Display::default() {
        let monitors = display.monitors();
        if let Some(monitor) = monitors
            .item(0)
            .and_then(|obj| obj.downcast::<Monitor>().ok())
        {
            let geo = monitor.geometry();
            window.set_default_width(geo.width());
            window.set_size_request(geo.width(), -1);
        }
    }
    window.set_hexpand(true);
    window.set_halign(Align::Fill);
}

