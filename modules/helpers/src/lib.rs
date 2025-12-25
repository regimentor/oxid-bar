use std::{collections::HashMap, fs, sync::Mutex};

use anyhow::Result;
use once_cell::sync::Lazy;

const HICOLOR: &str = "/usr/share/icons";
const PIXMAPS: &str = "/usr/share/pixmaps";

static ICON_CACHE: Lazy<Mutex<HashMap<String, Vec<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Получает пути к иконкам по имени иконки или классу приложения
pub fn icon_fetcher(app_class_name: &str) -> Result<Vec<String>> {
    if let Some(cached) = ICON_CACHE
        .lock()
        .ok()
        .and_then(|cache| cache.get(app_class_name).cloned())
    {
        return Ok(cached);
    }

    let mut result: Vec<String> = vec![];
    walker(HICOLOR, app_class_name, &mut result)?;
    walker(PIXMAPS, app_class_name, &mut result)?;

    if let Ok(mut cache) = ICON_CACHE.lock() {
        cache.insert(app_class_name.to_string(), result.clone());
    }

    Ok(result)
}

fn walker(path: &str, filename: &str, result: &mut Vec<String>) -> Result<()> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walker(path.to_str().unwrap(), filename, result)?;
        } else if let Some(name) = path.file_stem()
            && name == filename
        {
            let path = path.to_str();
            if let Some(path) = path {
                result.push(path.to_string());
            }
        }
    }

    Ok(())
}

