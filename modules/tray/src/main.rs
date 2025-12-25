use tray::Tray;

#[tokio::main]
async fn main() -> zbus::Result<()> {
    println!("Инициализация Tray...");
    let tray = match Tray::new().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Ошибка создания Tray: {e}");
            return Err(e);
        }
    };

    println!("Получение элементов трея...\n");
    let items = match tray.get_items().await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Ошибка получения элементов трея: {e}");
            return Err(e);
        }
    };

    println!("Повторный запрос элементов трея...\n");
    let _items_cached = match tray.get_items().await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Ошибка получения элементов трея: {e}");
            return Err(e);
        }
    };

    if items.is_empty() {
        println!("Элементы трея не найдены.");
        return Ok(());
    }

    println!("Найдено элементов: {}\n", items.len());
    println!("{}", "=".repeat(80));

    for (idx, item) in items.iter().enumerate() {
        println!("\n[Элемент {}]", idx + 1);
        println!("{}", "-".repeat(80));
        println!("ID:              {}", item.id);
        println!("Заголовок:       {}", item.title);
        println!("Статус:          {:?}", item.status);
        println!("Категория:       {}", item.category);
        println!("Window ID:       {}", item.window_id);
        println!("Является меню:   {}", item.is_menu);

        // Информация об иконке
        println!("\nИконка:");
        if let Some(ref icon_name) = item.icon.name {
            println!("  Имя:           {}", icon_name);
        } else {
            println!("  Имя:           (не указано)");
        }
        if let Some((width, height, data)) = &item.icon.pixmap {
            println!("  Pixmap:        {}x{} ({} байт)", width, height, data.len());
        } else {
            println!("  Pixmap:        (нет данных)");
        }
        if let Some(ref attention) = item.icon.attention_name {
            println!("  Attention:     {}", attention);
        }
        if let Some(ref overlay) = item.icon.overlay_name {
            println!("  Overlay:       {}", overlay);
        }
        if !item.icon.icon_paths.is_empty() {
            println!("  Пути к иконкам:");
            for (idx, path) in item.icon.icon_paths.iter().enumerate() {
                println!("    [{}] {}", idx + 1, path);
            }
        } else {
            println!("  Пути к иконкам: (не найдено)");
        }

        // Информация о меню
        if let Some(ref menu_path) = item.menu_path {
            println!("\nМеню:");
            println!("  Путь:         {}", menu_path);
        }

        // Информация о tooltip
        println!("\nTooltip:");
        let tooltip = &item.tooltip;
        if !tooltip.title().is_empty() {
            println!("  Заголовок:    {}", tooltip.title());
        }
        if !tooltip.description().is_empty() {
            println!("  Описание:     {}", tooltip.description());
        }
        if !tooltip.icon_name().is_empty() {
            println!("  Иконка:       {}", tooltip.icon_name());
        }
        if !tooltip.icon_pixmap().is_empty() {
            println!("  Pixmap:       {} вариантов", tooltip.icon_pixmap().len());
        }

        println!("{}", "-".repeat(80));
    }

    println!("\n{}", "=".repeat(80));
    println!("\nТестирование завершено.");

    Ok(())
}
