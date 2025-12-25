use gtk4::{Label, prelude::*};
use lang::get_layout_flag;

/// Компонент для отображения текущей раскладки клавиатуры
pub struct LangComponent {
    label: Label,
}

impl LangComponent {
    /// Создает новый компонент lang
    pub fn new(label: Label) -> Self {
        label.add_css_class("lang");
        label.set_halign(gtk4::Align::End);
        label.set_margin_end(12);
        Self { label }
    }

    /// Обновляет отображаемую раскладку
    pub fn update(&self) {
        match get_layout_flag() {
            Ok(flag) => {
                self.label.set_text(&flag);
            }
            Err(e) => {
                self.label.set_text("—");
                logger::log_error("LangComponent", e);
            }
        }
    }
}

