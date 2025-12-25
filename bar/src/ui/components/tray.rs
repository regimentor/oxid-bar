use gtk4::{Box, Image, Label, GestureClick, PopoverMenu, gdk::Rectangle, prelude::*};
use gtk4::gio::{Menu, SimpleAction, SimpleActionGroup};
use gdk_pixbuf::Pixbuf;
use glib::MainContext;
use std::{rc::Rc, cell::RefCell, collections::HashMap};
use tray::{MenuNode, Tray, TrayItem};
use zvariant::{OwnedValue, Value};

use crate::config::BarConfig;

/// Компонент для отображения системного трея
pub struct TrayComponent {
    container: Box,
    root: Box,
    config: BarConfig,
    tray: Rc<RefCell<Tray>>,
    popovers: Rc<RefCell<HashMap<u64, PopoverMenu>>>,
    popover_counter: Rc<RefCell<u64>>,
}

impl TrayComponent {
    /// Создает новый компонент tray
    pub fn new(
        container: Box,
        root: Box,
        config: BarConfig,
        tray: Rc<RefCell<Tray>>,
    ) -> Self {
        container.add_css_class("tray");
        container.set_halign(gtk4::Align::End);
        container.set_margin_end(12);
        
        Self {
            container,
            root,
            config,
            tray,
            popovers: Rc::new(RefCell::new(HashMap::new())),
            popover_counter: Rc::new(RefCell::new(0)),
        }
    }

    /// Обновляет содержимое трея
    pub fn refresh(&self, items: &[TrayItem]) {
        self.clear_children();

        for item in items {
            if let Some(widget) = self.create_tray_item_widget(item) {
                self.container.append(&widget);
            }
        }
    }

    fn create_tray_item_widget(&self, item: &TrayItem) -> Option<gtk4::Widget> {
        let icon_widget = self.create_icon_widget(item)?;
        
        // Добавляем обработчик клика для показа контекстного меню
        if item.menu_path.is_some() {
            self.add_menu_handler(&icon_widget, item);
        }
        
        Some(icon_widget)
    }

    fn create_icon_widget(&self, item: &TrayItem) -> Option<gtk4::Widget> {
        // Приоритет 1: иконка из icon_paths
        if let Some(icon_path) = item.icon.icon_paths.first()
            .filter(|p| std::path::Path::new(p).exists())
        {
            return Some(self.create_image_from_path(icon_path, item));
        }

        // Приоритет 2: pixmap данные
        if let Some((width, height, data)) = &item.icon.pixmap {
            return Some(self.create_image_from_pixmap(*width, *height, data, item));
        }

        // Приоритет 3: fallback на первую букву
        Some(self.create_fallback_label(item))
    }

    fn create_image_from_path(&self, icon_path: &str, item: &TrayItem) -> gtk4::Widget {
        let image = Image::from_file(icon_path);
        image.set_pixel_size(self.config.icon_size);
        image.set_margin_end(4);
        image.set_tooltip_text(Some(&Self::get_tooltip_text(item)));
        image.upcast()
    }

    fn create_image_from_pixmap(
        &self,
        width: i32,
        height: i32,
        data: &[u8],
        item: &TrayItem,
    ) -> gtk4::Widget {
        // SNI спецификация определяет формат как ARGB32 (alpha, red, green, blue)
        // Нужно конвертировать в RGBA для GdkPixbuf
        let rowstride = width * 4;
        let expected_size = (rowstride * height) as usize;
        
        if data.len() < expected_size {
            logger::log_error("TrayComponent::create_image_from_pixmap", 
                format!("Invalid pixmap data size: expected {}, got {}", expected_size, data.len()));
            return self.create_fallback_label(item);
        }
        
        // Конвертируем ARGB32 в RGBA
        let mut rgba_data = Vec::with_capacity(expected_size);
        for chunk in data.chunks_exact(4) {
            // ARGB32: [A, R, G, B] -> RGBA: [R, G, B, A]
            let a = chunk[0];
            let r = chunk[1];
            let g = chunk[2];
            let b = chunk[3];
            rgba_data.extend_from_slice(&[r, g, b, a]);
        }
        
        let bytes = gtk4::glib::Bytes::from(&rgba_data[..]);
        let pixbuf = Pixbuf::from_bytes(
            &bytes,
            gdk_pixbuf::Colorspace::Rgb,
            true, // has_alpha = true
            8,
            width,
            height,
            rowstride,
        );
        
        let image = Image::from_pixbuf(Some(&pixbuf));
        image.set_pixel_size(self.config.icon_size);
        image.set_margin_end(4);
        image.set_tooltip_text(Some(&Self::get_tooltip_text(item)));
        image.upcast()
    }

    fn create_fallback_label(&self, item: &TrayItem) -> gtk4::Widget {
        let letter = Self::get_first_letter(item);
        let label = Label::new(Some(&letter));
        label.add_css_class("tray-item-letter");
        label.set_margin_end(4);
        label.set_tooltip_text(Some(&Self::get_tooltip_text(item)));
        label.upcast()
    }

    fn get_tooltip_text(item: &TrayItem) -> String {
        if !item.title.is_empty() {
            item.title.clone()
        } else if !item.tooltip.title().is_empty() {
            item.tooltip.title().to_string()
        } else {
            item.id.clone()
        }
    }

    fn get_first_letter(item: &TrayItem) -> String {
        if !item.title.is_empty() {
            item.title
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>())
                .unwrap_or_default()
        } else if !item.tooltip.title().is_empty() {
            item.tooltip.title()
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>())
                .unwrap_or_default()
        } else {
            item.id
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>())
                .unwrap_or_default()
        }
    }

    fn add_menu_handler(&self, widget: &gtk4::Widget, item: &TrayItem) {
        let widget_clone = widget.clone();
        let root_clone = self.root.clone();
        let item_clone = item.clone();
        let tray_clone = self.tray.clone();
        let popovers_clone = self.popovers.clone();
        let counter_clone = self.popover_counter.clone();
        
        let click = GestureClick::new();
        click.set_button(3); // Правая кнопка мыши
        click.connect_pressed(move |_gesture, _, _x, _y| {
            let widget_ref = widget_clone.clone();
            let root_ref = root_clone.clone();
            let item_ref = item_clone.clone();
            let tray_ref = tray_clone.clone();
            let popovers_ref = popovers_clone.clone();
            let counter_ref = counter_clone.clone();
            
            MainContext::default().spawn_local(async move {
                let item_clone = item_ref.clone();
                let menu_result = {
                    #[allow(clippy::await_holding_refcell_ref)]
                    tray_ref.borrow().get_item_menu(&item_clone).await
                };
                
                match menu_result {
                    Ok(Some(menu_node)) => {
                        // Создаем action group для обработки активации элементов меню
                        let action_group = SimpleActionGroup::new();
                        let item_ref_for_actions = item_ref.clone();
                        let tray_ref_for_actions = tray_ref.clone();
                        
                        // Регистрируем действия для всех элементов меню
                        Self::register_menu_actions(&menu_node, &action_group, 
                            item_ref_for_actions.clone(), tray_ref_for_actions.clone());
                        
                        // Привязываем action group к root виджету
                        root_ref.insert_action_group("tray", Some(&action_group));
                        
                        let popover = Self::build_popup_menu(&menu_node, &item_ref);
                        popover.add_css_class("tray-menu");
                        
                        // Устанавливаем фон программно для надежности
                        let style_context = popover.style_context();
                        style_context.add_class("tray-menu");
                        
                        popover.set_parent(&root_ref);
                        popover.set_has_arrow(false);
                        popover.set_autohide(true);
                        popover.set_can_focus(true);
                        
                        // Устанавливаем фон для содержимого popover
                        if let Some(child) = popover.child() {
                            child.add_css_class("tray-menu-content");
                        }
                        
                        // Позиционируем popover относительно виджета
                        if let Some((x, y)) = widget_ref.translate_coordinates(&root_ref, 0.0, 0.0) {
                            let widget_width = widget_ref.width();
                            let widget_height = widget_ref.height();
                            popover.set_pointing_to(Some(&Rectangle::new(
                                (x + (widget_width / 2) as f64) as i32,
                                (y + widget_height as f64) as i32,
                                1,
                                1,
                            )));
                        }
                        
                        // Сохраняем popover
                        let popover_id = {
                            let mut c = counter_ref.borrow_mut();
                            *c += 1;
                            *c
                        };
                        
                        popovers_ref.borrow_mut().insert(popover_id, popover.clone());
                        popover.popup();
                        
                        // Обработчик закрытия
                        let popover_id_cleanup = popover_id;
                        let popovers_cleanup = popovers_ref.clone();
                        popover.connect_closed(move |_| {
                            popovers_cleanup.borrow_mut().remove(&popover_id_cleanup);
                        });
                    }
                    Ok(None) => {
                        // Меню нет или пустое
                    }
                    Err(e) => {
                        logger::log_error(&format!("TrayComponent::get_menu({})", item_clone.id), e);
                    }
                }
            });
        });
        widget.add_controller(click);
    }

    fn build_popup_menu(node: &MenuNode, item: &TrayItem) -> PopoverMenu {
        let gio_menu = Self::build_gio_menu(node, item);
        PopoverMenu::from_model(Some(&gio_menu))
    }

    fn build_gio_menu(node: &MenuNode, item: &TrayItem) -> Menu {
        let menu = Menu::new();
        
        let children_count = node.children.len();
        
        for (index, child) in node.children.iter().enumerate() {
            let label = child
                .props
                .get("label")
                .and_then(|v| {
                    let val: Value = Value::from(v.clone());
                    <Value as TryInto<String>>::try_into(val).ok()
                })
                .unwrap_or_default();
            
            let item_type: String = child
                .props
                .get("type")
                .and_then(|v| {
                    let val: Value = Value::from(v.clone());
                    <Value as TryInto<String>>::try_into(val).ok()
                })
                .unwrap_or_default();
            
            if item_type == "separator" {
                // Пропускаем сепаратор, если он последний элемент
                if index == children_count - 1 {
                    continue;
                }
                // Создаем сепаратор как специальный элемент меню с уникальным action
                let separator_action = format!("separator.{}", child.id);
                let separator_item = gtk4::gio::MenuItem::new(Some(""), Some(&separator_action));
                menu.append_item(&separator_item);
            } else if !child.children.is_empty() {
                let submenu = Self::build_gio_menu(child, item);
                menu.append_submenu(Some(&label), &submenu);
            } else {
                let action_name = format!("tray.item.{}", child.id);
                
                // Проверяем, включен ли элемент
                let enabled = child
                    .props
                    .get("enabled")
                    .and_then(|v| {
                        let val: Value = Value::from(v.clone());
                        <Value as TryInto<bool>>::try_into(val).ok()
                    })
                    .unwrap_or(true);
                
                // Создаем элемент меню с поддержкой состояния enabled/disabled
                let menu_item = gtk4::gio::MenuItem::new(Some(&label), Some(&action_name));
                if !enabled {
                    menu_item.set_attribute_value("disabled", Some(&"true".into()));
                }
                menu.append_item(&menu_item);
            }
        }
        
        menu
    }
    
    fn register_menu_actions(
        node: &MenuNode,
        action_group: &SimpleActionGroup,
        item: TrayItem,
        tray: Rc<RefCell<Tray>>,
    ) {
        for child in &node.children {
            let child_id = child.id;
            let item_clone = item.clone();
            let tray_clone = tray.clone();
            
            // Проверяем тип элемента - регистрируем действия только для обычных элементов (не separator)
            let item_type: String = child
                .props
                .get("type")
                .and_then(|v| {
                    let val: Value = Value::from(v.clone());
                    <Value as TryInto<String>>::try_into(val).ok()
                })
                .unwrap_or_default();
            
            // Пропускаем разделители
            if item_type == "separator" {
                continue;
            }
            
            // Если есть подменю, регистрируем действия рекурсивно
            if !child.children.is_empty() {
                Self::register_menu_actions(child, action_group, item.clone(), tray.clone());
                continue;
            }
            
            // Создаем действие с уникальным именем, соответствующим имени в build_gio_menu
            let action_name = format!("item.{}", child_id);
            let action = SimpleAction::new(&action_name, None);
            let action_clone = action.clone();
            
            action.connect_activate(move |_, _| {
                let item_ref = item_clone.clone();
                let tray_ref = tray_clone.clone();
                let menu_id = child_id;
                
                MainContext::default().spawn_local(async move {
                    // Получаем DBusMenu proxy
                    if let Some(menu_path) = &item_ref.menu_path {
                        let menu_path_str = menu_path.as_str();
                        if menu_path_str != "/" && !menu_path_str.is_empty() {
                            let tray_borrow = tray_ref.borrow();
                            let connection = tray_borrow.connection();
                            let bus_name = item_ref.bus_name.clone();
                            
                            match tray::DBusMenuProxy::builder(connection)
                                .destination(bus_name.as_str())
                            {
                                Ok(builder) => {
                                    match builder.path(menu_path_str) {
                                        Ok(path_builder) => {
                                            match path_builder.build().await {
                                                Ok(menu_proxy) => {
                                                    // Отправляем событие активации (event_id = "clicked")
                                                    let timestamp = std::time::SystemTime::now()
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                        .unwrap_or_default()
                                                        .as_secs() as u32;
                                                    // Создаем пустой variant для data параметра
                                                    // DBusMenu Event data обычно пустой словарь для обычных кликов
                                                    let data = OwnedValue::from(HashMap::<String, OwnedValue>::new());
                                                    
                                                    if let Err(e) = menu_proxy.event(menu_id, "clicked", data, timestamp).await {
                                                        logger::log_error("TrayComponent::menu_action", 
                                                            format!("Failed to send event for item {}: {}", menu_id, e));
                                                    }
                                                }
                                                Err(e) => {
                                                    logger::log_error("TrayComponent::menu_action", 
                                                        format!("Failed to build menu proxy: {}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            logger::log_error("TrayComponent::menu_action", 
                                                format!("Failed to set path: {}", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    logger::log_error("TrayComponent::menu_action", 
                                        format!("Failed to create menu proxy builder: {}", e));
                                }
                            }
                        }
                    }
                });
            });
            
            action_group.add_action(&action_clone);
        }
    }

    fn clear_children(&self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
    }
}

