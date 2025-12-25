use gtk4::{Application, ApplicationWindow, Box, Label, Orientation, prelude::*};
use glib::{MainContext, timeout_add_local, ControlFlow};
use std::{rc::Rc, cell::RefCell, sync::mpsc, time::Duration};
use tray::Tray;

use crate::config::BarConfig;
use crate::ui::{load_css, setup_window};
use crate::ui::components::{WorkspacesComponent, TrayComponent, ClockComponent, LangComponent};
use crate::services::start_hyprland_event_listener;

/// Основная логика приложения
pub struct BarApp {
    config: BarConfig,
}

impl BarApp {
    pub fn new() -> Self {
        Self {
            config: BarConfig::new(),
        }
    }

    pub fn build_ui(&self, app: &Application) {
        load_css();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("OxidBar")
            .build();
        
        setup_window(&window, &self.config);

        let root = Box::new(Orientation::Horizontal, self.config.spacing);
        root.add_css_class("bar");
        root.set_hexpand(true);
        root.set_halign(gtk4::Align::Fill);

        // Workspaces
        let workspaces_box = Box::new(Orientation::Horizontal, self.config.spacing);
        let workspaces_component = Rc::new(RefCell::new(WorkspacesComponent::new(
            workspaces_box.clone(),
            self.config.clone(),
        )));
        root.append(&workspaces_box);

        // Spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        root.append(&spacer);

        // Tray
        let tray_box = Box::new(Orientation::Horizontal, 6);
        root.append(&tray_box);

        // Lang
        let lang_label = Label::new(None);
        let lang_component = Rc::new(RefCell::new(LangComponent::new(lang_label.clone())));
        root.append(&lang_label);

        // Clock
        let clock_label = Label::new(None);
        let clock_component = Rc::new(RefCell::new(ClockComponent::new(clock_label.clone())));
        root.append(&clock_label);

        window.set_child(Some(&root));
        window.present();

        // Инициализация компонентов
        workspaces_component.borrow().refresh();
        lang_component.borrow().update();
        clock_component.borrow().update();

        // Запуск Hyprland event listener
        let (tx, rx) = mpsc::channel();
        start_hyprland_event_listener(tx);

        // Инициализация tray
        let tray_box_clone = tray_box.clone();
        let root_clone = root.clone();
        let config_clone = self.config.clone();
        MainContext::default().spawn_local(async move {
            match Tray::new().await {
                Ok(tray) => {
                    let tray_rc = Rc::new(RefCell::new(tray));
                    let tray_component = TrayComponent::new(
                        tray_box_clone.clone(),
                        root_clone.clone(),
                        config_clone.clone(),
                        tray_rc.clone(),
                    );
                    Self::start_tray_updater(tray_box_clone, root_clone, tray_rc, tray_component, config_clone);
                }
                Err(e) => {
                    logger::log_error("TrayInitialization", e);
                }
            }
        });

        // Workspaces таймер - проверка событий Hyprland
        let workspaces_clone = workspaces_component.clone();
        let config_workspaces = self.config.clone();
        timeout_add_local(Duration::from_millis(config_workspaces.workspaces_check_interval_ms), move || {
            let mut refreshed = false;
            while rx.try_recv().is_ok() {
                refreshed = true;
            }
            if refreshed {
                workspaces_clone.borrow().refresh();
            }
            ControlFlow::Continue
        });

        // Lang таймер - обновление раскладки клавиатуры
        let lang_clone = lang_component.clone();
        let config_lang = self.config.clone();
        timeout_add_local(Duration::from_millis(config_lang.lang_update_interval_ms), move || {
            lang_clone.borrow().update();
            ControlFlow::Continue
        });

        // Clock таймер - обновление времени
        let clock_clone = clock_component.clone();
        let config_clock = self.config.clone();
        timeout_add_local(Duration::from_millis(config_clock.clock_update_interval_ms), move || {
            clock_clone.borrow().update();
            ControlFlow::Continue
        });
    }

    fn start_tray_updater(
        _tray_box: Box,
        _root: Box,
        tray: Rc<RefCell<Tray>>,
        tray_component: TrayComponent,
        config: BarConfig,
    ) {
        let tray_clone = tray.clone();
        let component = Rc::new(RefCell::new(tray_component));
        let component_clone = component.clone();
        
        timeout_add_local(Duration::from_secs(config.tray_update_interval_secs), move || {
            let tray_ref = tray_clone.clone();
            let component_ref = component_clone.clone();
            
            MainContext::default().spawn_local(async move {
                let items = match tray_ref.borrow().get_items().await {
                    Ok(items) => items,
                    Err(e) => {
                        logger::log_error("TrayUpdater", e);
                        return;
                    }
                };
                component_ref.borrow_mut().refresh(&items);
            });
            ControlFlow::Continue
        });
    }
}

