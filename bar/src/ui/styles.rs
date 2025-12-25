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
}

popover.tray-menu {
    background-color: rgba(17, 24, 39, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    padding: 4px;
}

popover.tray-menu contents {
    background-color: rgba(17, 24, 39, 0.8);
}

popover.tray-menu > box {
    background-color: rgba(17, 24, 39, 0.8);
}

popover.tray-menu menu {
    background-color: rgba(17, 24, 39, 0.8);
    color: #e5e7eb;
}

popover.tray-menu list {
    background-color: rgba(17, 24, 39, 0.8);
}

popover.tray-menu listview {
    background-color: rgba(17, 24, 39, 0.8);
}

popover.tray-menu menuitem {
    padding: 6px 12px;
    border-radius: 4px;
    color: #e5e7eb;
    background-color: transparent;
}

popover.tray-menu menuitem:hover {
    background-color: rgba(255, 255, 255, 0.1);
}

popover.tray-menu menuitem label {
    color: #e5e7eb;
}

popover.tray-menu .tray-menu-content {
    background-color: rgba(17, 24, 39, 0.8);
}

popover.tray-menu menuitem {
    padding: 6px 12px;
}

popover.tray-menu menuitem[action^="separator"] {
    padding: 0px;
    margin: 4px 8px;
    min-height: 1px;
    max-height: 1px;
    background-color: rgba(255, 255, 255, 0.2);
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
}

popover.tray-menu menuitem[action^="separator"]:hover {
    background-color: rgba(255, 255, 255, 0.2);
    border-top: 1px solid rgba(255, 255, 255, 0.2);
}

popover.tray-menu menuitem[action^="separator"] > box {
    min-height: 1px;
    max-height: 1px;
    padding: 0px;
    margin: 0px;
}

popover.tray-menu menuitem[action^="separator"] label {
    opacity: 0;
    min-height: 0px;
    max-height: 0px;
    padding: 0px;
    margin: 0px;
    font-size: 0px;
}
"#;

/// Загружает CSS стили из файла или использует встроенные стили по умолчанию
pub fn load_css() {
    let provider = CssProvider::new();
    
    // Пытаемся загрузить из файла
    let css_path = Path::new("bar/resources/styles.css");
    let css_content = if css_path.exists() {
        match fs::read_to_string(css_path) {
            Ok(content) => {
                logger::log_info("Loaded CSS from file", css_path.display());
                content
            }
            Err(e) => {
                logger::log_error("Failed to read CSS file", e);
                DEFAULT_CSS.to_string()
            }
        }
    } else {
        logger::log_error("Failed to read CSS file", "File not found");
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

