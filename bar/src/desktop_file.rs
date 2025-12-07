use derive_more::Display;
use ini::Ini;

const DESKTOP_FILE_DIRS: [&str; 3] = [
    "/usr/share/applications",
    "/usr/local/share/applications",
    "~/.local/share/applications",
];

#[derive(Debug, Display, Clone)]
#[display("DesktopFile {{ name: {name}, exec: {exec}, icon: {icon:?} }}")]
pub struct DesktopFile {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
}

impl DesktopFile {
    pub fn load(app_class_name: &str) -> anyhow::Result<Option<Self>> {
        let Some(file_path) = Self::find(app_class_name) else {
            println!("Desktop file for '{}' not found", app_class_name);
            return Ok(None);
        };
        println!("Loading desktop file from: {}", file_path);
        let content = Ini::load_from_file(&file_path)?;
        let name = content
            .get_from(Some("Desktop Entry"), "Name")
            .unwrap_or("")
            .to_string();
        let exec = content
            .get_from(Some("Desktop Entry"), "Exec")
            .unwrap_or("")
            .to_string();
        let icon = content
            .get_from(Some("Desktop Entry"), "Icon")
            .map(|s| s.to_string());

        Ok(Some(DesktopFile { name, exec, icon }))
    }

    fn find(app_class_name: &str) -> Option<String> {
        for dir in DESKTOP_FILE_DIRS.iter() {
            let path = format!("{}/{}.desktop", dir, app_class_name);
            let path_lower = format!("{}/{}.desktop", dir, app_class_name.to_lowercase());
            if std::path::Path::new(&path_lower).exists() {
                return Some(path_lower);
            }
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
        None
    }
}
