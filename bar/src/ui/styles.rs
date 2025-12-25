use gtk4::{CssProvider, gdk::Display};
use std::fs;
use std::path::Path;

const DEFAULT_CSS: &str = r#"
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

popover.tray-menu {
    background-color: rgba(17, 24, 39, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    padding: 4px;
}

popover.tray-menu menu {
    background-color: transparent;
    color: #e5e7eb;
}

popover.tray-menu menuitem {
    padding: 6px 12px;
    border-radius: 4px;
}

popover.tray-menu menuitem:hover {
    background-color: rgba(255, 255, 255, 0.1);
}
"#;

/// Загружает CSS стили из файла или использует встроенные стили по умолчанию
pub fn load_css() {
    let provider = CssProvider::new();
    
    // Пытаемся загрузить из файла
    let css_path = Path::new("resources/styles.css");
    let css_content = if css_path.exists() {
        match fs::read_to_string(css_path) {
            Ok(content) => {
                eprintln!("Loaded CSS from file: {}", css_path.display());
                content
            }
            Err(e) => {
                eprintln!("Failed to read CSS file: {}. Using default CSS.", e);
                DEFAULT_CSS.to_string()
            }
        }
    } else {
        // Используем встроенные стили
        DEFAULT_CSS.to_string()
    };
    
    provider.load_from_data(&css_content);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

