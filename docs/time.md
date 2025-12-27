# time Module

Utilities for time handling and date/time formatting.

## Description

The module provides simple functions for formatting current local time using the `chrono` library.

## API

### Constants

```rust
pub const DEFAULT_PATTERN: &str = "%a %d %b %H:%M";
```

Default formatting pattern: day of week, day, month, hours:minutes (e.g., "Mon 15 Jan 14:30").

### Functions

#### format_local

Formats current local time according to the given pattern.

```rust
pub fn format_local(pattern: &str) -> String
```

**Parameters:**
- `pattern` - chrono formatting pattern string (e.g., "%Y-%m-%d %H:%M:%S")

**Returns:**
- `String` - formatted time

**Example:**
```rust
use time::format_local;

let date = format_local("%Y-%m-%d");
// "2024-01-15"

let time = format_local("%H:%M:%S");
// "14:30:45"
```

#### format_local_default

Formats current local time using the default pattern.

```rust
pub fn format_local_default() -> String
```

**Returns:**
- `String` - formatted time according to `DEFAULT_PATTERN`

**Example:**
```rust
use time::format_local_default;

let formatted = format_local_default();
// "Mon 15 Jan 14:30"
```

## Formatting Patterns

The module uses formatting patterns from the `chrono` library. Main patterns:

- `%Y` - year (4 digits)
- `%m` - month (01-12)
- `%d` - day of month (01-31)
- `%H` - hours (00-23)
- `%M` - minutes (00-59)
- `%S` - seconds (00-59)
- `%a` - abbreviated weekday name (Mon, Tue, etc.)
- `%b` - abbreviated month name (Jan, Feb, etc.)

Full list of patterns is available in [chrono documentation](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

## Tests

The module includes unit tests:

- `formats_with_custom_pattern` - checks formatting with custom pattern
- `formats_with_default_pattern` - checks formatting with default pattern

Run tests:
```bash
cargo test -p time
```

## Usage

```rust
use time::{format_local, format_local_default, DEFAULT_PATTERN};

// Using default pattern
let default = format_local_default();

// Using custom pattern
let custom = format_local("%Y-%m-%d %H:%M:%S");

// Using constant
let with_const = format_local(DEFAULT_PATTERN);
```

## Dependencies

- `chrono` - for time and date handling
