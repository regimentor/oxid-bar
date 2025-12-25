use anyhow::Result;
use hyprland::data::Devices;
use hyprland::shared::HyprData;

/// Получает текущую раскладку клавиатуры основной клавиатуры.
///
/// Возвращает `active_keymap` основной клавиатуры или ошибку, если
/// основная клавиатура не найдена.
pub fn get_current_layout() -> Result<String> {
    let devices = Devices::get()?;
    let keyboards = devices.keyboards;

    let main_keyboard = keyboards.iter().find(|k| k.main);

    let Some(main_keyboard) = main_keyboard else {
        return Err(anyhow::anyhow!("No main keyboard found"));
    };

    Ok(main_keyboard.active_keymap.clone())
}

/// Получает флаг для текущей раскладки клавиатуры.
///
/// Возвращает эмодзи флага в зависимости от раскладки:
/// - 🇷🇺 для русской раскладки
/// - 🇺🇸 для английской (US) раскладки
/// - исходную раскладку, если не удалось определить
pub fn get_layout_flag() -> Result<String> {
    let layout = get_current_layout()?;
    let layout_lower = layout.to_lowercase();

    // Проверяем различные варианты названий русской раскладки
    if layout_lower.contains("ru") || layout_lower.contains("russian") || layout_lower.contains("русск") {
        return Ok("🇷🇺".to_string());
    }

    // Проверяем различные варианты названий английской (US) раскладки
    if layout_lower.contains("us") || layout_lower.contains("english") || layout_lower.contains("en") {
        return Ok("🇺🇸".to_string());
    }

    // Если не удалось определить, возвращаем исходную раскладку
    Ok(layout)
}

