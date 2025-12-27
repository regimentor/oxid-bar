# lang Module

Module for working with keyboard layout in Hyprland.

## Description

The module provides functionality for getting the current keyboard layout of the main keyboard and converting it to a country flag emoji.

## API

### get_current_layout

Gets the current keyboard layout of the main keyboard.

```rust
pub fn get_current_layout() -> Result<String>
```

**Returns:**
- `Ok(String)` - current layout (e.g., "Russian", "English (US)")
- `Err` - if main keyboard is not found

**Example:**
```rust
use lang::get_current_layout;

match get_current_layout() {
    Ok(layout) => println!("Current layout: {}", layout),
    Err(e) => eprintln!("Error: {}", e),
}
```

### get_layout_flag

Gets the emoji flag for the current keyboard layout.

```rust
pub fn get_layout_flag() -> Result<String>
```

**Returns:**
- `Ok("🇷🇺")` - for Russian layout
- `Ok("🇺🇸")` - for English (US) layout
- `Ok(layout)` - original layout if detection failed

**Supported layout name variants:**

**Russian layout:**
- Contains "ru" (case-insensitive)
- Contains "russian" (case-insensitive)
- Contains "русск" (case-insensitive)

**English (US) layout:**
- Contains "us" (case-insensitive)
- Contains "english" (case-insensitive)
- Contains "en" (case-insensitive)

**Example:**
```rust
use lang::get_layout_flag;

match get_layout_flag() {
    Ok(flag) => println!("Current layout flag: {}", flag),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Usage

```rust
use lang::{get_current_layout, get_layout_flag};

// Get current layout
if let Ok(layout) = get_current_layout() {
    println!("Layout: {}", layout);
}

// Get layout flag
if let Ok(flag) = get_layout_flag() {
    println!("Flag: {}", flag);
}
```

## Features

1. **Main keyboard detection** - the module uses the `main` field from Hyprland data to identify the main keyboard
2. **Flexible layout detection** - check is performed by substrings in layout name (case-insensitive)
3. **Fallback** - if layout cannot be determined, returns the original name

## Dependencies

- `hyprland` - for getting keyboard data
- `anyhow` - for error handling

## Error Handling

The module returns `anyhow::Result`, allowing handling of various error types:

- Missing main keyboard
- Errors when getting data from Hyprland API
- Problems accessing system resources
