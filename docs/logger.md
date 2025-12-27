# logger Module

Logging module based on tracing.

## Description

The module provides a simple interface for logging using the `tracing` library. Uses `tracing_subscriber` for initialization and log formatting.

## API

### init

Initializes tracing subscriber for logging.

```rust
pub fn init()
```

Should be called once at the beginning of the application.

**Example:**
```rust
use logger::init;

fn main() {
    init();
    // ... rest of code
}
```

### log_error

Logs an error with context.

```rust
pub fn log_error(context: &str, error: impl std::fmt::Display)
```

**Parameters:**
- `context` - context where the error occurred (e.g., function or module name)
- `error` - error message

**Example:**
```rust
use logger::log_error;

log_error("MyFunction", "Failed to open file");
// Output: ERROR MyFunction: Failed to open file
```

### log_warning

Logs a warning with context.

```rust
pub fn log_warning(context: &str, message: impl std::fmt::Display)
```

**Parameters:**
- `context` - warning context
- `message` - warning message

**Example:**
```rust
use logger::log_warning;

log_warning("MyFunction", "Deprecated API usage");
// Output: WARN MyFunction: Deprecated API usage
```

### log_info

Logs an informational message with context.

```rust
pub fn log_info(context: &str, message: impl std::fmt::Display)
```

**Parameters:**
- `context` - message context
- `message` - informational message

**Example:**
```rust
use logger::log_info;

log_info("MyFunction", "Application started");
// Output: INFO MyFunction: Application started
```

### log_debug

Logs a debug message with context.

```rust
pub fn log_debug(context: &str, message: impl std::fmt::Display)
```

**Parameters:**
- `context` - debug message context
- `message` - debug message

**Example:**
```rust
use logger::log_debug;

log_debug("MyFunction", "Processing data");
// Output: DEBUG MyFunction: Processing data
```

## Usage

### Basic Usage

```rust
use logger::{init, log_info, log_error, log_warning, log_debug};

fn main() {
    init();
    
    log_info("main", "Application started");
    
    match some_operation() {
        Ok(_) => log_info("main", "Operation successful"),
        Err(e) => log_error("main", format!("Operation failed: {}", e)),
    }
}
```

### Usage in Modules

```rust
use logger::{log_info, log_error};

pub fn my_function() -> Result<()> {
    log_info("my_function", "Starting operation");
    
    // ... function code ...
    
    if let Err(e) = some_operation() {
        log_error("my_function", format!("Error: {}", e));
        return Err(e);
    }
    
    log_info("my_function", "Operation completed");
    Ok(())
}
```

## Log Levels

The module supports the following log levels (in descending order of importance):

1. **ERROR** - critical errors
2. **WARN** - warnings
3. **INFO** - informational messages
4. **DEBUG** - debug messages

## Output Format

By default, `tracing_subscriber::fmt::init()` outputs logs in the format:
```
LEVEL [timestamp] context: message
```

## Configuring Log Level

Log level can be configured via the `RUST_LOG` environment variable:

```bash
# Errors only
RUST_LOG=error cargo run

# Errors and warnings
RUST_LOG=warn cargo run

# All levels
RUST_LOG=debug cargo run

# Specific module
RUST_LOG=my_module=debug cargo run
```

## Dependencies

- `tracing` - structured logging library
- `tracing-subscriber` - subscriber for log formatting and output

## Recommendations

1. **Initialization** - call `init()` once at the beginning of `main()`
2. **Context** - use descriptive contexts (function name, module)
3. **Levels** - use appropriate log levels:
   - `log_error` - for errors that require attention
   - `log_warning` - for warnings about potential issues
   - `log_info` - for important application events
   - `log_debug` - for debug information
