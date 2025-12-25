use tray::{MenuNode, Tray};
use zvariant::Value;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    println!("Initializing Tray...");
    let tray = match Tray::new().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error creating Tray: {e}");
            return Err(e);
        }
    };

    println!("Getting tray items...\n");
    let items = match tray.get_items().await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Error getting tray items: {e}");
            return Err(e);
        }
    };

    println!("Requesting tray items again...\n");
    let _items_cached = match tray.get_items().await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Error getting tray items: {e}");
            return Err(e);
        }
    };

    if items.is_empty() {
        println!("No tray items found.");
        return Ok(());
    }

    println!("Found {} items\n", items.len());
    println!("{}", "=".repeat(80));

    for (idx, item) in items.iter().enumerate() {
        println!("\n[Item {}]", idx + 1);
        println!("{}", "-".repeat(80));
        println!("ID:              {}", item.id);
        println!("Title:           {}", item.title);
        println!("Status:          {:?}", item.status);
        println!("Category:        {}", item.category);
        println!("Window ID:       {}", item.window_id);
        println!("Is menu:         {}", item.is_menu);

        // Icon information
        println!("\nIcon:");
        if let Some(ref icon_name) = item.icon.name {
            println!("  Name:          {}", icon_name);
        } else {
            println!("  Name:          (not specified)");
        }
        if let Some((width, height, data)) = &item.icon.pixmap {
            println!("  Pixmap:        {}x{} ({} bytes)", width, height, data.len());
        } else {
            println!("  Pixmap:        (no data)");
        }
        if let Some(ref attention) = item.icon.attention_name {
            println!("  Attention:     {}", attention);
        }
        if let Some(ref overlay) = item.icon.overlay_name {
            println!("  Overlay:       {}", overlay);
        }
        if !item.icon.icon_paths.is_empty() {
            println!("  Icon paths:");
            for (idx, path) in item.icon.icon_paths.iter().enumerate() {
                println!("    [{}] {}", idx + 1, path);
            }
        } else {
            println!("  Icon paths:    (not found)");
        }

        // Menu information
        if let Some(ref menu_path) = item.menu_path {
            println!("\nMenu:");
            println!("  Path:         {}", menu_path);
            
            // Попытка получить структуру меню
            if let Ok(Some(menu)) = tray.get_item_menu(item).await {
                println!("  Menu structure:");
                fn dump(node: &MenuNode, depth: usize) {
                    let indent = "  ".repeat(depth);
                    let label = node
                        .props
                        .get("label")
                        .and_then(|v| {
                            let val: Value = Value::from(v.clone());
                            <Value as TryInto<String>>::try_into(val).ok()
                        })
                        .unwrap_or_default();
                    println!("{indent}- {} ({})", label, node.id);
                    for c in &node.children {
                        dump(c, depth + 1);
                    }
                }
                dump(&menu, 1);
            } else {
                println!("  (unable to fetch menu structure)");
            }
        }

        // Tooltip information
        println!("\nTooltip:");
        let tooltip = &item.tooltip;
        if !tooltip.title().is_empty() {
            println!("  Title:        {}", tooltip.title());
        }
        if !tooltip.description().is_empty() {
            println!("  Description:  {}", tooltip.description());
        }
        if !tooltip.icon_name().is_empty() {
            println!("  Icon:         {}", tooltip.icon_name());
        }
        if !tooltip.icon_pixmap().is_empty() {
            println!("  Pixmap:       {} variants", tooltip.icon_pixmap().len());
        }

        println!("{}", "-".repeat(80));
    }

    println!("\n{}", "=".repeat(80));
    println!("\nTesting completed.");

    Ok(())
}
