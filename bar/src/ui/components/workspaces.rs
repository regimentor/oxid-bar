use gtk4::{Box, Image, Label, Orientation, EventControllerMotion, GestureClick, PropagationPhase, prelude::*};
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland_workspaces::HyprWorkspaces;

use crate::config::BarConfig;

/// Компонент для отображения и управления workspace'ами Hyprland
pub struct WorkspacesComponent {
    container: Box,
    config: BarConfig,
}

impl WorkspacesComponent {
    /// Создает новый компонент workspace
    pub fn new(container: Box, config: BarConfig) -> Self {
        container.add_css_class("workspaces");
        Self { container, config }
    }

    /// Обновляет содержимое workspace'ов
    pub fn refresh(&self) {
        self.clear_children();

        match HyprWorkspaces::init() {
            Ok(workspaces) if !workspaces.map.is_empty() => {
                let mut entries: Vec<_> = workspaces.map.iter().collect();
                entries.sort_by_key(|(id, _)| *id);

                for (id, ws) in entries {
                    let ws_box = self.create_workspace_widget(*id, ws, workspaces.active_id);
                    self.container.append(&ws_box);
                }
            }
            Ok(_) => {
                let placeholder = Label::new(Some("Hyprland: no active workspaces"));
                placeholder.set_opacity(0.8);
                self.container.append(&placeholder);
            }
            Err(err) => {
                let error_label = Label::new(Some(&format!("Hyprland: request error ({err})")));
                error_label.set_opacity(0.8);
                self.container.append(&error_label);
            }
        }
    }

    fn create_workspace_widget(
        &self,
        id: i32,
        ws: &hyprland_workspaces::HyprWorkspace,
        active_id: Option<i32>,
    ) -> Box {
        let ws_box = Box::new(Orientation::Horizontal, 6);
        ws_box.add_css_class("workspace");
        ws_box.set_margin_start(4);
        ws_box.set_margin_end(4);
        
        if Some(id) == active_id {
            ws_box.add_css_class("active");
        }
        
        self.add_hover(&ws_box);
        self.add_click_switch(&ws_box, id);

        let title = Label::new(Some(&format!("{}:", id)));
        title.add_css_class("title");
        ws_box.append(&title);

        if ws.clients.is_empty() {
            let empty = Label::new(Some("—"));
            empty.set_opacity(0.6);
            ws_box.append(&empty);
        } else {
            for client in &ws.clients {
                let client_widget = self.create_client_widget(client);
                ws_box.append(&client_widget);
            }
        }

        ws_box
    }

    fn create_client_widget(&self, client: &hyprland_workspaces::HyprlandClient) -> gtk4::Widget {
        if let Some(icon_path) = client.icons.first() {
            let image = Image::from_file(icon_path);
            image.set_pixel_size(self.config.icon_size);
            image.set_margin_end(4);
            let tooltip = client
                .desktop_file
                .as_ref()
                .map(|df| df.name.clone())
                .unwrap_or_else(|| client.class.clone());
            image.set_tooltip_text(Some(&tooltip));
            image.upcast()
        } else {
            let fallback = Label::new(Some(&client.class));
            fallback.set_margin_end(6);
            fallback.set_opacity(0.8);
            fallback.upcast()
        }
    }

    fn add_click_switch(&self, widget: &Box, workspace_id: i32) {
        let click = GestureClick::new();
        click.set_propagation_phase(PropagationPhase::Capture);
        click.set_button(0);
        click.connect_released(move |_, _, _, _| {
            if let Err(err) = Dispatch::call(DispatchType::Workspace(
                WorkspaceIdentifierWithSpecial::Id(workspace_id),
            )) {
                logger::log_error("WorkspaceSwitch", err);
            }
        });
        widget.add_controller(click);
    }

    fn add_hover(&self, widget: &Box) {
        let motion = EventControllerMotion::new();
        motion.set_propagation_phase(PropagationPhase::Capture);
        motion.connect_enter(|controller, _, _| {
            if let Some(widget) = controller.widget() {
                widget.add_css_class("hover");
                widget.set_cursor_from_name(Some("pointer"));
            }
        });
        motion.connect_leave(|controller| {
            if let Some(widget) = controller.widget() {
                widget.remove_css_class("hover");
                widget.set_cursor_from_name(None);
            }
        });
        widget.add_controller(motion);
    }

    fn clear_children(&self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
    }
}

