use std::{collections::HashMap, fmt::Display, sync::Mutex};

use anyhow::Result;
use derive_more::Display as DeriveDisplay;
use helpers::icon_fetcher;
use hyprland::{
    data::{Clients, Workspace as ActiveWorkspace, Workspaces},
    shared::{HyprData, HyprDataActive},
};
use ini::Ini;
use once_cell::sync::Lazy;

pub type HyprWorkspacesMap = HashMap<i32, HyprWorkspace>;

#[derive(Debug, DeriveDisplay)]
#[display("class: [{class}] title({initial_title}): {title} (workspace: {workspace_id})")]
pub struct HyprlandClient {
    pub class: String,
    pub title: String,
    pub initial_title: String,
    pub workspace_id: i32,
    pub icons: Vec<String>,
    pub desktop_file: Option<DesktopFile>,
}

#[derive(Debug)]
pub struct HyprWorkspace {
    pub id: i32,
    pub monitor: String,
    pub monitor_id: Option<i128>,
    pub clients: Vec<HyprlandClient>,
}

impl Display for HyprWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Workspace {{id: {}, monitor({}): {}, clients: {}}}",
            self.id,
            self.monitor,
            if self.monitor_id.is_some() {
                self.monitor_id.unwrap().to_string()
            } else {
                "None".to_string()
            },
            self.clients.len()
        )
    }
}

#[derive(Debug)]
pub struct HyprWorkspaces {
    pub map: HyprWorkspacesMap,
    pub active_id: Option<i32>,
}

impl HyprWorkspaces {
    pub fn init() -> Result<Self> {
        let (ws_map, active_id) = Self::get_workspaces()?;
        Ok(HyprWorkspaces {
            map: ws_map,
            active_id,
        })
    }

    fn get_workspaces() -> Result<(HyprWorkspacesMap, Option<i32>)> {
        let mut hypr_ws: HashMap<i32, HyprWorkspace> = HashMap::new();

        let workspaces = Workspaces::get()?;
        for ws_i in workspaces.iter() {
            let workspace = HyprWorkspace {
                id: ws_i.id,
                monitor: ws_i.monitor.clone(),
                monitor_id: ws_i.monitor_id,
                clients: Vec::new(),
            };

            hypr_ws.insert(ws_i.id, workspace);
        }

        let clients = Clients::get()?;
        for client in clients.iter() {
            let desktop_file = DesktopFile::load(&client.class)?;
            let icons = if let Some(df) = &desktop_file {
                if let Some(icon_name) = &df.icon {
                    icon_fetcher(icon_name)?
                } else {
                    icon_fetcher(&client.class)?
                }
            } else {
                icon_fetcher(&client.class)?
            };
            let hypr_client = HyprlandClient {
                class: client.class.clone(),
                title: client.title.clone(),
                initial_title: client.initial_title.clone(),
                workspace_id: client.workspace.id,
                icons,
                desktop_file,
            };

            if let Some(ws) = hypr_ws.get_mut(&client.workspace.id) {
                ws.clients.push(hypr_client);
            }
        }

        let active_id = ActiveWorkspace::get_active()
            .map(|ws| ws.id)
            .map_err(|e| println!("Не удалось получить активный рабочий стол: {e}"))
            .ok();

        Ok((hypr_ws, active_id))
    }
}

const DESKTOP_FILE_DIRS: [&str; 3] = [
    "/usr/share/applications",
    "/usr/local/share/applications",
    "~/.local/share/applications",
];

static DESKTOP_CACHE: Lazy<Mutex<HashMap<String, Option<DesktopFile>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, DeriveDisplay, Clone)]
#[display("DesktopFile {{ name: {name}, exec: {exec}, icon: {icon:?} }}")]
pub struct DesktopFile {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
}

impl DesktopFile {
    pub fn load(app_class_name: &str) -> Result<Option<Self>> {
        if let Some(cached) = DESKTOP_CACHE
            .lock()
            .ok()
            .and_then(|cache| cache.get(app_class_name).cloned())
        {
            return Ok(cached);
        }

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

        let desktop = Some(DesktopFile { name, exec, icon });

        if let Ok(mut cache) = DESKTOP_CACHE.lock() {
            cache.insert(app_class_name.to_string(), desktop.clone());
        }

        Ok(desktop)
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

