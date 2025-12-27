# audio Module

Module for working with audio via PulseAudio backend.

## Description

The module provides functionality for monitoring and working with audio devices via PulseAudio. The module can be used as a library or as a standalone application.

## Structure

```
modules/audio/
├── src/
│   ├── lib.rs                    # Public module API
│   ├── main.rs                   # Entry point for standalone application
│   └── backend/
│       ├── mod.rs                # Backend module
│       └── pulse/                # PulseAudio backend
│           ├── mod.rs
│           ├── client.rs         # PulseAudio client
│           ├── device_info.rs    # Device information
│           ├── listen_pulse_backend.rs # PulseAudio event listener
│           ├── output_info.rs    # Output information
│           └── start_listening.rs # Start listening
```

## Usage as Library

### Public API

```rust
pub mod backend;
```

The module exports only the `backend` module, which contains the PulseAudio backend.

### Start Listening

```rust
use audio::backend::pulse::start_listening;

// Start listening to PulseAudio events
start_listening()?;
```

## Usage as Standalone Application

The module can be run as a separate application for monitoring audio events.

```bash
# Run standalone application
cargo run -p audio
```

The application:
1. Initializes logging
2. Starts PulseAudio listener in a separate thread
3. Waits for termination signal (Ctrl+C)

## Backend: PulseAudio

### Client

Client for working with PulseAudio.

```rust
pub struct Client {
    // ...
}
```

**Methods:**
- `new() -> Self` - creates a new client
- `start_listening(&mut self) -> Result<()>` - starts listening to events

### OutputInfo

Information about audio output (sink or sink input).

```rust
pub struct OutputInfo {
    pub index: u32,          // Output index
    pub sink: u32,           // Sink index
    pub client: Option<u32>, // Client index (for sink input)
    pub mute: bool,          // Whether output is muted
    pub name: String,        // Output name
    pub app_name: String,    // Application name
    pub volume_level: u32,   // Volume level (0-100)
}
```

**Methods:**
- `from_sink_input_info(input: &SinkInputInfo) -> OutputInfo` - creates from SinkInputInfo
- `from_sink_info(sink: &SinkInfo) -> OutputInfo` - creates from SinkInfo

### DeviceInfo

Audio device information.

See `device_info.rs` for implementation details.

## PulseAudio Events

The module listens to the following PulseAudio events:

- Sink state changes (output devices)
- Sink input state changes (applications)
- Volume changes
- Mute state changes
- Device add/remove

## Usage

### Basic Usage (Library)

```rust
use audio::backend::pulse::start_listening;

fn main() -> anyhow::Result<()> {
    logger::init();
    
    // Start listening to PulseAudio events
    start_listening()?;
    
    // Application continues...
    Ok(())
}
```

### Getting Output Information

```rust
use audio::backend::pulse::output_info::OutputInfo;

// OutputInfo is created automatically when handling PulseAudio events
// and contains information about current output state
```

## Features

1. **Async processing** - events are processed in a separate thread
2. **Automatic updates** - the module automatically tracks changes in PulseAudio
3. **Error handling** - uses `anyhow::Result` for error handling

## Dependencies

- `libpulse-binding` - Rust bindings for PulseAudio
- `anyhow` - for error handling
- `logger` - for logging

## Requirements

The module requires:
- Installed PulseAudio
- Access to PulseAudio server (usually via user session)

## Error Handling

The module uses `anyhow::Result` for error handling. Main error types:

- PulseAudio connection errors
- Errors when getting device information
- Event processing errors

## Usage Examples

### Monitoring Volume Changes

The module automatically tracks volume changes and can be used to update UI in real-time.

### Getting Application Information

The module provides information about applications using audio, including their names and volume levels.

### Device Management

The module can be extended for device management (volume change, mute/unmute, etc.).
