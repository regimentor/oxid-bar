use gtk4::{Label, prelude::*};
use time_utils::format_local_default;

/// Компонент для отображения времени
pub struct ClockComponent {
    label: Label,
}

impl ClockComponent {
    /// Создает новый компонент clock
    pub fn new(label: Label) -> Self {
        label.add_css_class("clock");
        label.set_halign(gtk4::Align::End);
        Self { label }
    }

    /// Обновляет отображаемое время
    pub fn update(&self) {
        self.label.set_text(&format_local_default());
    }
}

