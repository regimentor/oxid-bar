use helpers::icon_fetcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::{proxy, Error as ZbusError};
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
    fn icon_pixmap(&self) -> zbus::Result<IconPixmap>;
    #[zbus(property)]
    fn attention_icon_pixmap(&self) -> zbus::Result<IconPixmap>;
    #[zbus(property)]
    fn overlay_icon_pixmap(&self) -> zbus::Result<IconPixmap>;

    #[zbus(property)]
    fn menu(&self) -> zbus::Result<OwnedObjectPath>;
    #[zbus(property)]
    fn item_is_menu(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn window_id(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn tool_tip(&self) -> zbus::Result<ToolTip>;
}

#[proxy(interface = "com.canonical.dbusmenu")]
pub trait DBusMenu {
    fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: Vec<&str>,
    ) -> zbus::Result<GetLayoutResult>;
    
    fn get_property(
        &self,
        id: i32,
        name: &str,
    ) -> zbus::Result<OwnedValue>;
    
    fn event(
        &self,
        id: i32,
        event_id: &str,
        data: OwnedValue,
        timestamp: u32,
    ) -> zbus::Result<()>;
    
    fn about_to_show(
        &self,
        id: i32,
    ) -> zbus::Result<bool>;
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

#[derive(Debug, Clone)]
pub struct MenuNode {
    pub id: i32,
    pub props: HashMap<String, OwnedValue>,
    pub children: Vec<MenuNode>,
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
    
    /// Получить ссылку на DBus соединение
    pub fn connection(&self) -> &zbus::Connection {
        &self.connection
    }

    pub async fn get_items(&self) -> zbus::Result<Vec<TrayItem>> {
        let watcher = StatusNotifierWatcherProxy::new(&self.connection).await?;
        let registered_items = watcher.registered_status_notifier_items().await?;

        let mut items = Vec::new();

        for raw_item in registered_items {
            // BUS_NAME/OBJECT_PATH или только BUS_NAME
            let (bus_name, object_path) = match raw_item.split_once('/') {
                Some((bus, path)) => (bus.to_string(), format!("/{path}")),
                None => (raw_item.clone(), "/StatusNotifierItem".to_string()),
            };

            let item_proxy = match self.create_item_proxy(bus_name.clone(), object_path.clone()).await {
                Ok(proxy) => proxy,
                Err(e) => {
                    logger::log_error("Tray::create_proxy", format!("{raw_item}: {e}"));
                    continue;
                }
            };

            let item = match Self::fetch_item_data(&item_proxy, bus_name.clone(), object_path.clone()).await {
                Ok(item) => item,
                Err(e) => {
                    logger::log_error("Tray::fetch_item_data", format!("{raw_item}: {e}"));
                    continue;
                }
            };

            items.push(item);
        }

        Ok(items)
    }


    async fn create_item_proxy(
        &self,
        bus_name: String,
        object_path: String,
    ) -> zbus::Result<StatusNotifierItemProxy<'static>> {
        // Используем Box::leak для создания 'static reference из owned String
        // Это безопасно, так как bus_name будет храниться в TrayItem
        let bus_name_static: &'static str = Box::leak(bus_name.into_boxed_str());
        StatusNotifierItemProxy::builder(&self.connection)
            .destination(bus_name_static)?
            .path(object_path)?
            .build()
            .await
    }

    async fn fetch_item_data(item_proxy: &StatusNotifierItemProxy<'_>, bus_name: String, object_path: String) -> zbus::Result<TrayItem> {
        // Читаем основные свойства с логированием ошибок
        let id = item_proxy.id().await.unwrap_or_else(|e| {
            logger::log_error("Tray::fetch_item_data::id", &e);
            String::new()
        });
        let title = item_proxy.title().await.unwrap_or_else(|e| {
            logger::log_error("Tray::fetch_item_data::title", &e);
            String::new()
        });
        let status_str = item_proxy.status().await.unwrap_or_else(|e| {
            logger::log_error("Tray::fetch_item_data::status", &e);
            String::new()
        });
        let category = item_proxy.category().await.unwrap_or_else(|e| {
            logger::log_error("Tray::fetch_item_data::category", &e);
            String::new()
        });
        
        let icon_name = item_proxy.icon_name().await.ok();
        let attention_icon_name = item_proxy.attention_icon_name().await.ok();
        let overlay_icon_name = item_proxy.overlay_icon_name().await.ok();
        
        // Читаем pixmap свойства
        let icon_pixmap_prop = item_proxy.icon_pixmap().await.ok();
        let attention_icon_pixmap = item_proxy.attention_icon_pixmap().await.ok();
        let overlay_icon_pixmap = item_proxy.overlay_icon_pixmap().await.ok();
        
        let menu_path = item_proxy.menu().await.ok();
        
        // Вспомогательная функция для проверки, является ли ошибка отсутствием свойства
        fn is_missing_property_error(e: &ZbusError) -> bool {
            let err_str = e.to_string();
            err_str.contains("No such property") 
                || err_str.contains("UnknownProperty") 
                || err_str.contains("InvalidArgs")
                || err_str.contains("Property") && err_str.contains("was not found")
        }
        
        // Опциональные свойства - не логируем ошибки отсутствующих свойств
        let is_menu = item_proxy.item_is_menu().await.unwrap_or_else(|e| {
            if !is_missing_property_error(&e) {
                logger::log_error("Tray::fetch_item_data::item_is_menu", &e);
            }
            false
        });
        
        let window_id = item_proxy.window_id().await.unwrap_or_else(|e| {
            // Не логируем ошибки отсутствующих свойств или неправильного типа (некоторые приложения используют другой тип)
            if !is_missing_property_error(&e) && !e.to_string().contains("incorrect type") {
                logger::log_error("Tray::fetch_item_data::window_id", &e);
            }
            0
        });
        
        let tooltip = item_proxy.tool_tip().await.unwrap_or_else(|e| {
            if !is_missing_property_error(&e) {
                logger::log_error("Tray::fetch_item_data::tool_tip", &e);
            }
            ToolTip::new(String::new(), Vec::new(), String::new(), String::new())
        });

        // Приоритет: icon_pixmap property > tooltip.icon_pixmap
        let icon_pixmap = if let Some(ref pixmaps) = icon_pixmap_prop {
            if !pixmaps.is_empty() {
                pixmaps.first().cloned()
            } else {
                None
            }
        } else {
            None
        }.or_else(|| {
            if !tooltip.icon_pixmap().is_empty() {
                tooltip.icon_pixmap().first().cloned()
            } else {
                None
            }
        });

        // Получаем пути к иконкам с обработкой ошибок
        let icon_paths = if let Some(ref name) = icon_name {
            icon_fetcher(name).unwrap_or_else(|e| {
                logger::log_error("Tray::fetch_item_data::icon_fetcher", format!("icon_name={}: {}", name, e));
                Vec::new()
            })
        } else if !id.is_empty() {
            icon_fetcher(&id).unwrap_or_else(|e| {
                logger::log_error("Tray::fetch_item_data::icon_fetcher", format!("id={}: {}", id, e));
                Vec::new()
            })
        } else if !title.is_empty() {
            icon_fetcher(&title).unwrap_or_else(|e| {
                logger::log_error("Tray::fetch_item_data::icon_fetcher", format!("title={}: {}", title, e));
                Vec::new()
            })
        } else {
            Vec::new()
        };

        // Используем attention_icon_pixmap если статус NeedsAttention
        let final_pixmap = if status_str == "NeedsAttention" {
            attention_icon_pixmap
                .and_then(|pixmaps| pixmaps.first().cloned())
                .or(icon_pixmap)
        } else {
            icon_pixmap
        };
        
        // overlay_icon_pixmap можно использовать для отображения overlay иконки
        // Пока оставляем как есть, можно добавить поддержку позже
        let _ = overlay_icon_pixmap;

        let icon = TrayIcon {
            name: icon_name,
            pixmap: final_pixmap,
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

    pub async fn get_item_menu(
        &self,
        item: &TrayItem,
    ) -> zbus::Result<Option<MenuNode>> {
        // 1) Проверить menu_path
        let menu_path = match &item.menu_path {
            Some(path) => {
                let path_str = path.as_str();
                if path_str == "/" || path_str.is_empty() {
                    return Ok(None);
                }
                path_str
            }
            None => return Ok(None),
        };

        // 2) Создать DBusMenuProxy
        let menu_proxy = match DBusMenuProxy::builder(&self.connection)
            .destination(item.bus_name.as_str())?
            .path(menu_path)?
            .build()
            .await
        {
            Ok(proxy) => proxy,
            Err(e) => {
                logger::log_error("Tray::get_item_menu::create_proxy", format!("{}: {}", item.id, e));
                return Ok(None);
            }
        };

        // 3) Вызвать GetLayout(0, -1, [])
        let (_revision, layout_tuple) = match menu_proxy.get_layout(0, -1, vec![]).await {
            Ok(result) => result,
            Err(e) => {
                logger::log_error("Tray::get_item_menu::get_layout", format!("{}: {}", item.id, e));
                return Ok(None);
            }
        };

        // 4) Распарсить layout через parse_layout_tuple
        match parse_layout_tuple(layout_tuple) {
            Ok(node) => Ok(Some(node)),
            Err(e) => {
                logger::log_error("Tray::get_item_menu::parse_layout", format!("{}: {}", item.id, e));
                Ok(None)
            }
        }
    }
}

// Helper types для парсинга layout
// PropsDict должен быть HashMap для правильной десериализации a{sv} (словарь)
type PropsDict = HashMap<String, OwnedValue>;
// layout node: (i32, a{sv}, a(layout))
type LayoutTuple = (i32, PropsDict, Vec<OwnedValue>);

// Тип для возвращаемого значения GetLayout: (revision: u32, layout: layout_node)
type GetLayoutResult = (u32, LayoutTuple);

fn parse_layout_tuple(tuple: LayoutTuple) -> Result<MenuNode, zvariant::Error> {
    let (id, props, children_array) = tuple;

    // Рекурсивно разобрать детей
    let mut children = Vec::new();
    for child_value in children_array {
        // Преобразуем OwnedValue в LayoutTuple для рекурсивного парсинга
        let value: Value = Value::from(child_value);
        match <Value as TryInto<LayoutTuple>>::try_into(value) {
            Ok(child_tuple) => {
                match parse_layout_tuple(child_tuple) {
                    Ok(child_node) => children.push(child_node),
                    Err(e) => {
                        logger::log_error("Tray::parse_layout_tuple::child", e);
                        // Продолжаем парсинг остальных детей, не паникуем
                    }
                }
            }
            Err(e) => {
                logger::log_error("Tray::parse_layout_tuple::convert", e);
                // Продолжаем парсинг остальных детей, не паникуем
            }
        }
    }

    Ok(MenuNode {
        id,
        props,
        children,
    })
}
