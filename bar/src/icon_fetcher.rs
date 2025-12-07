use std::fs;

const HICOLOR: &str = "/usr/share/icons";
const PIXMAPS: &str = "/usr/share/pixmaps";

pub fn icon_fetcher(app_class_name: &str) -> anyhow::Result<Vec<String>> {
    // This is a placeholder for the icon fetching logic.
    // In a real implementation, this function would contain code to fetch and return icons.
    println!("Icon fetcher function called.");

    let mut result: Vec<String> = vec![];
    walker(HICOLOR, app_class_name, &mut result)?;
    walker(PIXMAPS, app_class_name, &mut result)?;

    Ok(result)
}

fn walker(path: &str, filename: &str, result: &mut Vec<String>) -> anyhow::Result<()> {
    // Placeholder for directory walking logic

    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walker(path.to_str().unwrap(), filename, result)?;
        } else if let Some(name) = path.file_stem() && name == filename {
                let path = path.to_str();
                if let Some(path) = path {
                    result.push(path.to_string());
                }
        }
    }

    Ok(())
}
