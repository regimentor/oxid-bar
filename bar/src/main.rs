use gtk4::{
    Align, Application, ApplicationWindow, Box, CssProvider, EventControllerMotion, GestureClick,
    Image, Label, Orientation, PropagationPhase, StyleContext,
    gdk::{Display, Monitor},
    glib::{self, MainContext},
    prelude::*,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use gdk_pixbuf::Pixbuf;
use hyprland::{
    dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial},
    event_listener::EventListener,
};
use hyprland_workspaces::HyprWorkspaces;
use lang::get_layout_flag;
use std::{rc::Rc, cell::RefCell, sync::mpsc, thread, time::Duration};
use time_utils::format_local_default;
use tray::Tray;

fn main() {
    let app = Application::builder()
        .application_id("rs.regimentor.oxidbar")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    load_css();
    let bar_height = 32;

    let window = ApplicationWindow::builder()
        .application(app)
        .title("OxidBar")
        .default_height(bar_height)
        .build();
    window.set_hexpand(true);
    window.set_halign(Align::Fill);

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
    set_full_width(&window);

    window.set_decorated(false);
    window.set_resizable(false);

    let root = Box::new(Orientation::Horizontal, 12);
    root.add_css_class("bar");
    root.set_hexpand(true);
    root.set_halign(Align::Fill);

    let workspaces_box = Box::new(Orientation::Horizontal, 12);
    workspaces_box.add_css_class("workspaces");
    root.append(&workspaces_box);

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    root.append(&spacer);

    let tray_box = Box::new(Orientation::Horizontal, 6);
    tray_box.add_css_class("tray");
    tray_box.set_halign(Align::End);
    tray_box.set_margin_end(12);
    root.append(&tray_box);

    let lang_label = Label::new(None);
    lang_label.add_css_class("lang");
    lang_label.set_halign(Align::End);
    lang_label.set_margin_end(12);
    root.append(&lang_label);

    let clock_label = Label::new(None);
    clock_label.add_css_class("clock");
    clock_label.set_halign(Align::End);
    root.append(&clock_label);

    window.set_child(Some(&root));
    window.present();

    refresh_hypr_content(&workspaces_box);
    update_lang(&lang_label);
    update_clock(&clock_label);

    let (tx, rx) = mpsc::channel();
    start_hypr_event_listener(tx);

    let tray_box_clone = tray_box.clone();
    MainContext::default().spawn_local(async move {
        match Tray::new().await {
            Ok(tray) => {
                let tray_rc = Rc::new(RefCell::new(tray));
                start_tray_updater(tray_box_clone, tray_rc);
            }
            Err(e) => {
                eprintln!("Ошибка инициализации трея: {e}");
            }
        }
    });

    let workspaces_clone = workspaces_box.clone();
    let lang_clone = lang_label.clone();
    let clock_clone = clock_label.clone();
    glib::timeout_add_local(Duration::from_millis(100), move || {
        let mut refreshed = false;
        while rx.try_recv().is_ok() {
            refreshed = true;
        }
        if refreshed {
            refresh_hypr_content(&workspaces_clone);
        }
        update_lang(&lang_clone);
        update_clock(&clock_clone);
        glib::ControlFlow::Continue
    });
}

fn refresh_hypr_content(container: &Box) {
    clear_children(container);

    match HyprWorkspaces::init() {
        Ok(workspaces) if !workspaces.map.is_empty() => {
            let mut entries: Vec<_> = workspaces.map.iter().collect();
            entries.sort_by_key(|(id, _)| *id);

            for (id, ws) in entries {
                let ws_box = Box::new(Orientation::Horizontal, 6);
                ws_box.add_css_class("workspace");
                ws_box.set_margin_start(4);
                ws_box.set_margin_end(4);
                if Some(*id) == workspaces.active_id {
                    ws_box.add_css_class("active");
                }
                add_hover(&ws_box);
                add_click_switch(&ws_box, *id);

                let title = Label::new(Some(&format!("{}:", id)));
                title.add_css_class("title");
                ws_box.append(&title);

                if ws.clients.is_empty() {
                    let empty = Label::new(Some("—"));
                    empty.set_opacity(0.6);
                    ws_box.append(&empty);
                } else {
                    for client in &ws.clients {
                        let mut img_opt = None;
                        if let Some(icon_path) = client.icons.first() {
                            img_opt = Some(Image::from_file(icon_path));
                        }

                        if let Some(icon) = img_opt {
                            icon.set_pixel_size(20);
                            icon.set_margin_end(4);
                            let tooltip = client
                                .desktop_file
                                .as_ref()
                                .map(|df| df.name.clone())
                                .unwrap_or_else(|| client.class.clone());
                            icon.set_tooltip_text(Some(&tooltip));
                            ws_box.append(&icon);
                        } else {
                            let fallback = Label::new(Some(&client.class));
                            fallback.set_margin_end(6);
                            fallback.set_opacity(0.8);
                            ws_box.append(&fallback);
                        }
                    }
                }

                container.append(&ws_box);
            }
        }
        Ok(_) => {
            let placeholder = Label::new(Some("Hyprland: нет активных рабочих столов"));
            placeholder.set_opacity(0.8);
            container.append(&placeholder);
        }
        Err(err) => {
            let error_label = Label::new(Some(&format!("Hyprland: ошибка запроса ({err})")));
            error_label.set_opacity(0.8);
            container.append(&error_label);
        }
    };
}

const CSS: &str = r#"
window {
    background-color: transparent;
}

.bar {
    background-color: rgba(17, 24, 39, 0.8);
    padding: 6px 12px;
}

.workspaces {
    color: #e5e7eb;
}

.workspace {
    border: 1px solid rgba(255, 255, 255, 0.2);
    background-color: rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    padding: 4px 6px;
    transition: background-color 120ms ease, border-color 120ms ease, box-shadow 120ms ease;
}

.workspace.active {
    border-color: rgba(125, 211, 252, 0.7);
    background-color: rgba(125, 211, 252, 0.16);
    box-shadow: 0 0 0 1px rgba(125, 211, 252, 0.25);
}

.workspace.hover {
    border-color: rgba(255, 255, 255, 0.4);
    background-color: rgba(255, 255, 255, 0.12);
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.12);
}

.workspace .title {
    font-weight: 600;
    margin-right: 4px;
}

.lang {
    color: #e5e7eb;
    font-weight: 600;
    letter-spacing: 0.3px;
}

.clock {
    color: #e5e7eb;
    font-weight: 600;
    letter-spacing: 0.3px;
}

.tray {
    color: #e5e7eb;
}

.tray-item-letter {
    color: #e5e7eb;
    font-weight: 600;
    font-size: 14px;
    min-width: 20px;
    min-height: 20px;
    border-radius: 4px;
    background-color: rgba(255, 255, 255, 0.1);
    padding: 2px 6px;
    display: flex;
    align-items: center;
    justify-content: center;
}
"#;

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(CSS);

    if let Some(display) = Display::default() {
        StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn clear_children(container: &Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn add_click_switch(widget: &Box, workspace_id: i32) {
    let click = GestureClick::new();
    click.set_propagation_phase(PropagationPhase::Capture);
    click.set_button(0);
    click.connect_released(move |_, _, _, _| {
        if let Err(err) = Dispatch::call(DispatchType::Workspace(
            WorkspaceIdentifierWithSpecial::Id(workspace_id),
        )) {
            eprintln!("Не удалось перейти на рабочий стол {workspace_id}: {err}");
        }
    });
    widget.add_controller(click);
}

fn add_hover(widget: &Box) {
    let motion = EventControllerMotion::new();
    motion.set_propagation_phase(PropagationPhase::Capture);
    motion.connect_enter(|controller, _, _| {
        if let Some(widget) = controller.widget() {
            widget.add_css_class("hover");
            widget.set_cursor_from_name(Some("pointer"));
        }
    });
    motion.connect_leave(|controller| {
        if let Some(widget) = controller.widget() {
            widget.remove_css_class("hover");
            widget.set_cursor_from_name(None);
        }
    });
    widget.add_controller(motion);
}

fn update_lang(label: &Label) {
    match get_layout_flag() {
        Ok(flag) => {
            label.set_text(&flag);
        }
        Err(e) => {
            label.set_text("—");
            eprintln!("Ошибка получения раскладки: {}", e);
        }
    }
}

fn update_clock(label: &Label) {
    label.set_text(&format_local_default());
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

fn refresh_tray_content(container: &Box, items: &[tray::TrayItem]) {
    clear_children(container);

    for item in items {
        let mut icon_widget: Option<gtk4::Widget> = None;

        // Приоритет 1: иконка из icon_paths
        if let Some(icon_path) = item.icon.icon_paths.first()
            .filter(|p| std::path::Path::new(p).exists())
        {
            let image = Image::from_file(icon_path);
            image.set_pixel_size(20);
            image.set_margin_end(4);
            let tooltip = if !item.title.is_empty() {
                item.title.clone()
            } else if !item.tooltip.title().is_empty() {
                item.tooltip.title().to_string()
            } else {
                item.id.clone()
            };
            image.set_tooltip_text(Some(&tooltip));
            icon_widget = Some(image.upcast());
        }

        // Приоритет 2: pixmap данные
        if icon_widget.is_none() 
            && let Some((width, height, data)) = &item.icon.pixmap
        {
            let bytes = glib::Bytes::from(&data[..]);
            let pixbuf = Pixbuf::from_bytes(&bytes, gdk_pixbuf::Colorspace::Rgb, false, 8, *width, *height, width * 4);
            let image = Image::from_pixbuf(Some(&pixbuf));
            image.set_pixel_size(20);
            image.set_margin_end(4);
            let tooltip = if !item.title.is_empty() {
                item.title.clone()
            } else if !item.tooltip.title().is_empty() {
                item.tooltip.title().to_string()
            } else {
                item.id.clone()
            };
            image.set_tooltip_text(Some(&tooltip));
            icon_widget = Some(image.upcast());
        }

        // Приоритет 3: fallback на первую букву
        if icon_widget.is_none() {
            let letter = if !item.title.is_empty() {
                item.title
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>())
                    .unwrap_or_default()
            } else if !item.tooltip.title().is_empty() {
                item.tooltip.title().chars()
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>())
                    .unwrap_or_default()
            } else {
                item.id
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>())
                    .unwrap_or_default()
            };

            let label = Label::new(Some(&letter));
            label.add_css_class("tray-item-letter");
            label.set_margin_end(4);
            let tooltip = if !item.title.is_empty() {
                item.title.clone()
            } else if !item.tooltip.title().is_empty() {
                item.tooltip.title().to_string()
            } else {
                item.id.clone()
            };
            label.set_tooltip_text(Some(&tooltip));
            icon_widget = Some(label.upcast());
        }

        if let Some(widget) = icon_widget {
            container.append(&widget);
        }
    }
}


fn start_tray_updater(tray_box: Box, tray: Rc<RefCell<Tray>>) {
    let tray_box_clone = tray_box.clone();
    let tray_clone = tray.clone();
    glib::timeout_add_local(Duration::from_secs(1), move || {
        let tray_box_ref = tray_box_clone.clone();
        let tray_ref = tray_clone.clone();
        MainContext::default().spawn_local({
            #[allow(clippy::await_holding_refcell_ref)]
            async move {
                let items = match tray_ref.borrow().get_items().await {
                Ok(items) => items,
                Err(e) => {
                    eprintln!("Ошибка получения элементов трея: {e}");
                    return;
                }
            };
                refresh_tray_content(&tray_box_ref, &items);
            }
        });
        glib::ControlFlow::Continue
    });
}

fn start_hypr_event_listener(tx: mpsc::Sender<()>) {
    thread::spawn(move || {
        let mut listener = EventListener::new();

        let send = |desc: &str, tx: &mpsc::Sender<()>| {
            if let Err(err) = tx.send(()) {
                eprintln!("Hyprland listener send error ({desc}): {err}");
            }
        };

        let tx_ws = tx.clone();
        listener.add_workspace_changed_handler(move |_| send("workspace_changed", &tx_ws));

        let tx_added = tx.clone();
        listener.add_workspace_added_handler(move |_| send("workspace_added", &tx_added));

        let tx_deleted = tx.clone();
        listener.add_workspace_deleted_handler(move |_| send("workspace_deleted", &tx_deleted));

        let tx_moved = tx.clone();
        listener.add_workspace_moved_handler(move |_| send("workspace_moved", &tx_moved));

        let tx_renamed = tx.clone();
        listener.add_workspace_renamed_handler(move |_| send("workspace_renamed", &tx_renamed));

        let tx_open = tx.clone();
        listener.add_window_opened_handler(move |_| send("window_opened", &tx_open));

        let tx_close = tx.clone();
        listener.add_window_closed_handler(move |_| send("window_closed", &tx_close));

        if let Err(err) = listener.start_listener() {
            eprintln!("Hyprland listener error: {err}");
        }
    });
}
