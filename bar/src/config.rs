/// Конфигурация для Bar приложения
#[derive(Debug, Clone)]
pub struct BarConfig {
    /// Высота бара в пикселях
    pub height: i32,
    /// Интервал проверки событий Hyprland для workspaces в миллисекундах
    pub workspaces_check_interval_ms: u64,
    /// Интервал обновления раскладки клавиатуры в миллисекундах
    pub lang_update_interval_ms: u64,
    /// Интервал обновления часов в миллисекундах
    pub clock_update_interval_ms: u64,
    /// Интервал обновления tray в секундах
    pub tray_update_interval_secs: u64,
    /// Размер иконок в пикселях
    pub icon_size: i32,
    /// Отступы между элементами
    pub spacing: i32,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            height: 32,
            workspaces_check_interval_ms: 100,
            lang_update_interval_ms: 200,
            clock_update_interval_ms: 1000,
            tray_update_interval_secs: 1,
            icon_size: 20,
            spacing: 12,
        }
    }
}

impl BarConfig {
    /// Создает новую конфигурацию с значениями по умолчанию
    pub fn new() -> Self {
        Self::default()
    }
}

