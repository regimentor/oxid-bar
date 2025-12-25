use helpers::icon_fetcher;
use serde::{Deserialize, Serialize};
use zbus::proxy;
use zvariant::{OwnedObjectPath, OwnedValue, Type, Value};

#[proxy(
    interface = "org.kde.StatusNotifierWatcher",
    default_service = "org.kde.StatusNotifierWatcher",
    default_path = "/StatusNotifierWatcher"
)]
pub trait StatusNotifierWatcher {
    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> zbus::Result<Vec<String>>;
}

#[derive(Debug, Clone, Type, Serialize, Deserialize)]
pub struct ToolTip {
    icon_name: String,
    icon_pixmap: Vec<(i32, i32, Vec<u8>)>,
    title: String,
    description: String,
}

type IconPixmap = Vec<(i32, i32, Vec<u8>)>;
type ToolTipTuple = (String, IconPixmap, String, String);
impl ToolTip {
    pub fn new(
        icon_name: String,
        icon_pixmap: IconPixmap,
        title: String,
        description: String,
    ) -> Self {
        Self {
            icon_name,
            icon_pixmap,
            title,
            description,
        }
    }

    pub fn icon_name(&self) -> &str {
        &self.icon_name
    }

    pub fn icon_pixmap(&self) -> &IconPixmap {
        &self.icon_pixmap
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

impl std::convert::TryFrom<OwnedValue> for ToolTip {
    type Error = zvariant::Error;

    fn try_from(owned: OwnedValue) -> Result<Self, Self::Error> {
        let v: Value = Value::from(owned);

        let (icon_name, icon_pixmap, title, description): ToolTipTuple = v.try_into()?;

        Ok(Self {
            icon_name,
            icon_pixmap,
            title,
            description,
        })
    }
}

#[proxy(interface = "org.kde.StatusNotifierItem")]
pub trait StatusNotifierItem {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn category(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn attention_icon_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn overlay_icon_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn menu(&self) -> zbus::Result<OwnedObjectPath>;
    #[zbus(property)]
    fn item_is_menu(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn window_id(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn tool_tip(&self) -> zbus::Result<ToolTip>;


}

#[derive(Debug, Clone)]
pub struct TrayIcon {
    pub name: Option<String>,
    pub pixmap: Option<(i32, i32, Vec<u8>)>,
    pub attention_name: Option<String>,
    pub overlay_name: Option<String>,
    pub icon_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayItemStatus {
    Passive,
    Active,
    NeedsAttention,
}

impl From<&str> for TrayItemStatus {
    fn from(s: &str) -> Self {
        match s {
            "Active" => TrayItemStatus::Active,
            "NeedsAttention" => TrayItemStatus::NeedsAttention,
            _ => TrayItemStatus::Passive,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrayItem {
    pub id: String,
    pub title: String,
    pub status: TrayItemStatus,
    pub category: String,
    pub icon: TrayIcon,
    pub tooltip: ToolTip,
    pub menu_path: Option<OwnedObjectPath>,
    pub is_menu: bool,
    pub window_id: u32,
    pub bus_name: String,
    pub object_path: String,
}

pub struct Tray {
    connection: zbus::Connection,
}

impl Tray {
    pub async fn new() -> zbus::Result<Self> {
        let connection = zbus::Connection::session().await?;
        Ok(Self {
            connection,
        })
    }

    pub async fn get_items(&self) -> zbus::Result<Vec<TrayItem>> {
        let watcher = StatusNotifierWatcherProxy::new(&self.connection).await?;
        let registered_items = watcher.registered_status_notifier_items().await?;

        let mut items = Vec::new();

        for raw_item in registered_items {
            // BUS_NAME/OBJECT_PATH
            let (bus_name, object_path) = match raw_item.split_once('/') {
                Some((bus, path)) => (bus, format!("/{path}")),
                None => continue,
            };

            let item_proxy = match self.create_item_proxy(bus_name, object_path.clone()).await {
                Ok(proxy) => proxy,
                Err(e) => {
                    eprintln!("Ошибка создания прокси для {raw_item}: {e}");
                    continue;
                }
            };

            let item = match Self::fetch_item_data(&item_proxy, bus_name, object_path.clone()).await {
                Ok(item) => item,
                Err(e) => {
                    eprintln!("Ошибка получения данных элемента {raw_item}: {e}");
                    continue;
                }
            };

            items.push(item);
        }

        Ok(items)
    }


    async fn create_item_proxy<'a>(
        &self,
        bus_name: &'a str,
        object_path: String,
    ) -> zbus::Result<StatusNotifierItemProxy<'a>> {
        StatusNotifierItemProxy::builder(&self.connection)
            .destination(bus_name)?
            .path(object_path)?
            .build()
            .await
    }

    async fn fetch_item_data(item_proxy: &StatusNotifierItemProxy<'_>, bus_name: &str, object_path: String) -> zbus::Result<TrayItem> {
        let id = item_proxy.id().await.unwrap_or_default();
        let title = item_proxy.title().await.unwrap_or_default();
        let status_str = item_proxy.status().await.unwrap_or_default();
        let category = item_proxy.category().await.unwrap_or_default();
        let icon_name = item_proxy.icon_name().await.ok();
        let attention_icon_name = item_proxy.attention_icon_name().await.ok();
        let overlay_icon_name = item_proxy.overlay_icon_name().await.ok();
        let menu_path = item_proxy.menu().await.ok();
        let is_menu = item_proxy.item_is_menu().await.unwrap_or(false);
        let window_id = item_proxy.window_id().await.unwrap_or(0);
        let tooltip = item_proxy.tool_tip().await.unwrap_or_else(|_| {
            ToolTip::new(String::new(), Vec::new(), String::new(), String::new())
        });

        let icon_pixmap = if !tooltip.icon_pixmap().is_empty() {
            tooltip.icon_pixmap().first().cloned()
        } else {
            None
        };

        let icon_paths = if let Some(ref name) = icon_name {
            icon_fetcher(name).unwrap_or_default()
        } else if !id.is_empty() {
            icon_fetcher(&id).unwrap_or_default()
        } else if !title.is_empty() {
            icon_fetcher(&title).unwrap_or_default()
        } else {
            Vec::new()
        };

        let icon = TrayIcon {
            name: icon_name,
            pixmap: icon_pixmap,
            attention_name: attention_icon_name,
            overlay_name: overlay_icon_name,
            icon_paths,
        };

        Ok(TrayItem {
            id,
            title,
            status: TrayItemStatus::from(status_str.as_str()),
            category,
            icon,
            tooltip,
            menu_path,
            is_menu,
            window_id,
            bus_name: bus_name.to_string(),
            object_path,
        })
    }
}
