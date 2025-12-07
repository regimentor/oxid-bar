use std::{collections::HashMap, fmt::Display};

use derive_more::Display as DeriveDisplay;
use hyprland::{
    data::{Clients, Workspaces},
    shared::HyprData,
};

use crate::{desktop_file::DesktopFile, icon_fetcher::icon_fetcher};

type HyprWorkspacesMap = HashMap<i32, HyprWorkspace>;

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
}

impl HyprWorkspaces {
    pub fn init() -> anyhow::Result<Self> {
        let ws_map = Self::get_workspaces()?;
        Ok(HyprWorkspaces { map: ws_map })
    }

    fn get_workspaces() -> anyhow::Result<HyprWorkspacesMap> {
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
            let icons = icon_fetcher(&client.class)?;
            let desktop_file = DesktopFile::load(&client.class)?;
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

        Ok(hypr_ws)
    }
}
