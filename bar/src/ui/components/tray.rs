use gtk4::{Box, Image, Label, GestureClick, PopoverMenu, gdk::Rectangle, prelude::*};
use gtk4::gio::Menu;
use gdk_pixbuf::Pixbuf;
use glib::MainContext;
use std::{rc::Rc, cell::RefCell, collections::HashMap};
use tray::{MenuNode, Tray, TrayItem};
use zvariant::Value;

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
        let bytes = gtk4::glib::Bytes::from(data);
        let pixbuf = Pixbuf::from_bytes(
            &bytes,
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            width,
            height,
            width * 4,
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
                        let popover = Self::build_popup_menu(&menu_node);
                        popover.add_css_class("tray-menu");
                        popover.set_parent(&root_ref);
                        popover.set_has_arrow(false);
                        popover.set_autohide(true);
                        popover.set_can_focus(true);
                        
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

    fn build_popup_menu(node: &MenuNode) -> PopoverMenu {
        let gio_menu = Self::build_gio_menu(node);
        PopoverMenu::from_model(Some(&gio_menu))
    }

    fn build_gio_menu(node: &MenuNode) -> Menu {
        let menu = Menu::new();
        
        for child in &node.children {
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
                menu.append(Some(&label), None::<&str>);
            } else if !child.children.is_empty() {
                let submenu = Self::build_gio_menu(child);
                menu.append_submenu(Some(&label), &submenu);
            } else {
                let action_name = format!("item.{}", child.id);
                menu.append(Some(&label), Some(&action_name));
            }
        }
        
        menu
    }

    fn clear_children(&self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
    }
}

