# План реализации интерфейса для PulseAudio виджета в баре

## Overview

Implement a UI component for displaying and controlling audio (global volume and per-application volume) in the OxidBar status bar. UI elements live in the `audio` module, and all backend interaction logic is implemented inside the UI module.

## Architecture

- **UI module:** `modules/audio/src/ui/` - hosts all UI components and the backend interaction logic
- **Backend module:** `modules/audio/src/backend/pulse/` - extended to support volume events and control commands
- **Integration:** `build_ui` in the `audio` module accepts bar containers and wires the widget

## Goals

1. Extend `AudioEvent` to cover global and per-application volume events
2. Extend `AudioCmd` with volume control commands
3. Create the UI module in `modules/audio/src/ui/` with a `build_ui` entry point
4. Implement a bar icon and a volume control popup
5. Implement backend interaction logic inside the UI module
6. Integrate the widget into the bar via `build_ui`

## Tasks

### 1. Extend AudioEvent for volume events

**File:** `modules/audio/src/backend/pulse/client.rs`

- [ ] Extend `AudioEvent` with the following variants:
  ```rust
  pub enum AudioEvent {
      // Global volume (sink)
      GlobalVolumeChanged {
          sink_index: u32,
          volume: u32,  // 0-100
          muted: bool,
      },
      // Application volume (sink input)
      AppVolumeChanged {
          sink_input_index: u32,
          volume: u32,  // 0-100
          muted: bool,
          app_name: String,
      },
      // Applications list updated
      AppsListUpdated {
          apps: Vec<OutputInfo>,
      },
      // Global volume received (initial state)
      GlobalVolumeReceived {
          sink_index: u32,
          volume: u32,
          muted: bool,
      },
      // Application volume received (initial state)
      AppVolumeReceived {
          sink_input_index: u32,
          volume: u32,  // 0-100
          muted: bool,
          app_name: String,
      },
  }
  ```

**Expected result:** `AudioEvent` supports all required UI events

---

### 2. Extend AudioCmd with control commands

**File:** `modules/audio/src/backend/pulse/client.rs`

- [ ] Extend `AudioCmd` with the following variants:
  ```rust
  pub enum AudioCmd {
      // Existing commands
      AddOutput(OutputInfo),
      ChangeOutput(u32, OutputInfo),
      
      // New control commands
      // Set global volume
      SetGlobalVolume {
          sink_index: u32,
          volume: u32,  // 0-100
      },
      // Toggle global mute
      ToggleGlobalMute {
          sink_index: u32,
      },
      // Set application volume
      SetAppVolume {
          sink_input_index: u32,
          volume: u32,  // 0-100
      },
      // Toggle application mute
      ToggleAppMute {
          sink_input_index: u32,
      },
      // Request current global volume
      RequestGlobalVolume {
          sink_index: Option<u32>,  // None = default sink
      },
      // Request list of applications with their volume
      RequestAppsList,
  }
  ```

**Expected result:** `AudioCmd` supports all volume control commands

---

### 3. Extend backend to handle commands and emit events

**File:** `modules/audio/src/backend/pulse/listen_pulse_backend.rs`

- [ ] Update `lesten_pulse_backend` to accept `event_tx: mpsc::Sender<AudioEvent>`
- [ ] Add subscription handling for SINK events (global volume)
- [ ] On sink info updates, send `AudioEvent::GlobalVolumeChanged`
- [ ] On sink input info updates, send `AudioEvent::AppVolumeChanged`
- [ ] On initial load, send `AudioEvent::GlobalVolumeReceived` and `AudioEvent::AppVolumeReceived`

**File:** `modules/audio/src/backend/pulse/client.rs`

- [ ] Update `Client::start_listening` to handle new commands:
  - `SetGlobalVolume` - set volume via the PulseAudio API
  - `ToggleGlobalMute` - toggle mute via the PulseAudio API
  - `SetAppVolume` - set application volume
  - `ToggleAppMute` - toggle application mute
  - `RequestGlobalVolume` - request current volume and send an event
  - `RequestAppsList` - request the applications list and send an event

**Expected result:** Backend handles control commands and sends events

---

### 4. Create the UI module structure

**File:** `modules/audio/src/ui/mod.rs`

- [ ] Add a `ui` module in `modules/audio/src/lib.rs`: `pub mod ui;`
- [ ] Create `modules/audio/src/ui/mod.rs` exporting:
  ```rust
  pub mod widget;
  pub mod popup;
  
  pub use widget::build_ui;
  ```

**Expected result:** UI module structure is created

---

### 5. Create the bar widget (icon)

**File:** `modules/audio/src/ui/widget.rs`

**Structure:**
- [ ] Create an `AudioWidget` struct with fields:
  - `icon_container: Box` - icon container in the bar
  - `icon_label: Label` - label displaying icon or volume
  - `client: Rc<RefCell<Client>>` - backend client
  - `current_global_volume: u32` - current global volume
  - `is_global_muted: bool` - mute state
  - `popup: Option<AudioPopup>` - popup reference (optional)

**Methods:**
- [ ] `new(icon_container: Box, client: Rc<RefCell<Client>>) -> Self` - create widget
  - Create a `Label` for the icon
  - Add CSS class `audio-widget-icon`
  - Add the label to the container
  - Initialize default values
- [ ] `update_icon(&self)` - update icon
  - Display a speaker or muted icon based on mute state
  - Optionally include volume percentage
- [ ] `handle_event(&mut self, event: AudioEvent)` - handle events
  - Handle `GlobalVolumeChanged` - update global volume
  - Handle `GlobalVolumeReceived` - set initial state
  - Call `update_icon()` after updates
- [ ] `start_event_listener(&self)` - start event processing
  - Read `client.event_channels.rx` in a separate thread or via a timer
  - Call `handle_event` for each event
  - Use `MainContext` to update UI from another thread

**Expected result:** Widget shows the bar icon and updates on changes

---

### 6. Create a popup for volume control

**File:** `modules/audio/src/ui/popup.rs`

**Structure:**
- [ ] Create an `AudioPopup` struct with fields:
  - `popup_window: Popover` or `Box` - popup container
  - `client: Rc<RefCell<Client>>` - backend client
  - `global_volume_scale: Scale` - global volume slider
  - `global_mute_button: Button` - mute or unmute button
  - `apps_list: ListBox` or `Box` - list of applications with their volume
  - `apps: Vec<AppControl>` - list of app controls

**AppControl struct:**
- [ ] Create an `AppControl` struct for per-app volume control:
  ```rust
  struct AppControl {
      app_name: String,
      sink_input_index: u32,
      volume_scale: Scale,
      mute_button: Button,
      volume_label: Label,
  }
  ```

**AudioPopup methods:**
- [ ] `new(root: &Box, client: Rc<RefCell<Client>>) -> Self` - create popup
  - Create a `Popover` or `Box` with CSS class `audio-popup`
  - Create the global volume slider
  - Create the mute or unmute button
  - Create a container for the applications list
  - Set up event handlers
- [ ] `update_global_volume(&self, volume: u32, muted: bool)` - update global volume
  - Set the slider value
  - Update the mute button state
- [ ] `update_apps_list(&mut self, apps: Vec<OutputInfo>)` - update applications list
  - Create or update an `AppControl` for each app
  - Add sliders and buttons for each app
- [ ] `handle_event(&mut self, event: AudioEvent)` - handle events
  - Update UI when events arrive
- [ ] `on_global_volume_changed(&self, value: f64)` - slider change handler
  - Send `AudioCmd::SetGlobalVolume`
- [ ] `on_global_mute_clicked(&self)` - mute button click handler
  - Send `AudioCmd::ToggleGlobalMute`
- [ ] `on_app_volume_changed(&self, sink_input_index: u32, value: f64)` - app volume change handler
  - Send `AudioCmd::SetAppVolume`
- [ ] `on_app_mute_clicked(&self, sink_input_index: u32)` - app mute click handler
  - Send `AudioCmd::ToggleAppMute`

**Expected result:** Popup allows controlling global volume and per-application volume

---

### 7. Implement build_ui

**File:** `modules/audio/src/ui/widget.rs`

- [ ] Create a public `build_ui` function:
  ```rust
  pub fn build_ui(
      icon_container: &gtk4::Box,
      root: &gtk4::Box,
  ) -> anyhow::Result<()>
  ```
- [ ] In the function:
  - Create `Client::new()`
  - Wrap the client in `Rc<RefCell<>>`
  - Create `AudioWidget` with the icon
  - Create `AudioPopup` with the popup
  - Connect the widget and popup (show popup when the icon is clicked)
  - Start the PulseAudio listener in a separate thread
  - Start event handling for the widget and popup
  - Request initial state via `AudioCmd::RequestGlobalVolume` and `AudioCmd::RequestAppsList`

**Expected result:** `build_ui` integrates the widget into the bar

---

### 8. Add dependencies to the audio module

**File:** `modules/audio/Cargo.toml`

- [ ] Add dependencies:
  - `gtk4 = "0.10.3"` - for UI components
  - `glib = "0.19"` - for MainContext and timers

**Expected result:** The audio module has all required UI dependencies

---

### 9. Integrate into the bar

**File:** `bar/src/app.rs`

- [ ] Add import: `use audio::ui::build_ui;`
- [ ] In `build_ui` after creating `root`:
  - Create an audio icon container: `let audio_icon_box = Box::new(Orientation::Horizontal, 0);`
  - Add the container to `root` at the desired location (for example, before Clock)
  - Call `audio::ui::build_ui(&audio_icon_box, &root)?;`

**Expected result:** Audio widget is integrated into the bar

---

### 10. Add CSS styles

**File:** `bar/resources/styles.css`

- [ ] Add styles for `.audio-widget-icon`:
  ```css
  .audio-widget-icon {
      margin: 0 6px;
      padding: 0 8px;
      font-size: 14px;
      cursor: pointer;
  }
  ```
- [ ] Add styles for `.audio-popup`:
  ```css
  .audio-popup {
      padding: 12px;
      min-width: 300px;
  }
  ```
- [ ] Add styles for popup controls

**Expected result:** Widget and popup have styling

---

### 11. Update documentation

**File:** `docs/audio.md`

- [ ] Add a UI module description
- [ ] Describe `build_ui` and its parameters
- [ ] Describe the `AudioEvent` and `AudioCmd` structures
- [ ] Add usage examples

**File:** `docs/bar.md`

- [ ] Add a description of audio widget integration
- [ ] Mention calling `audio::ui::build_ui`

**Expected result:** Documentation is up to date and describes the audio widget

---

## Implementation Details

### Event architecture

**Bidirectional flow:**
- **UI → Backend:** commands via `AudioCmd` (set volume, mute)
- **Backend → UI:** events via `AudioEvent` (volume changes, list updates)

### Data structure

**The bar widget displays:**
- A speaker or muted icon depending on mute state
- Optional volume percentage

**The popup displays:**
- A global volume slider (0-100%)
- A mute/unmute button for global volume
- A list of applications with:
  - Application name
  - Volume slider (0-100%)
  - Mute/unmute button

### PulseAudio API usage

**For getting volume:**
- Use `introspector.get_sink_info()` for global volume
- Use `introspector.get_sink_input_info_list()` for the applications list
- Subscribe to events via `ctx.subscribe(InterestMaskSet::SINK | InterestMaskSet::SINK_INPUT)`

**For setting volume:**
- Use `introspector.set_sink_volume_by_index()` for global volume
- Use `introspector.set_sink_input_volume_by_index()` for application volume
- Use `introspector.set_sink_mute_by_index()` and `introspector.set_sink_input_mute_by_index()` for mute

### UI updates

**Event-driven approach:**
- Backend sends events via `event_channels.tx`
- The UI module reads events via `event_channels.rx`
- Use `MainContext::default().invoke()` to update UI from another thread

### Error handling

- On PulseAudio connection errors, display "-" or hide the component
- Log errors via `logger::log_error()`
- Graceful degradation: the app should keep running even when audio is unavailable
- On volume set errors, log but do not panic

### Testing

- [ ] Verify the icon updates on volume changes
- [ ] Verify the icon updates on mute or unmute
- [ ] Verify popup open and close behavior
- [ ] Verify volume changes via sliders
- [ ] Verify mute or unmute via buttons
- [ ] Verify applications list updates
- [ ] Verify behavior when PulseAudio is unavailable
- [ ] Verify updates when switching audio devices

## Execution Order

1. **Backend:**
   - Extend `AudioEvent` and `AudioCmd`
   - Update the backend to handle commands and emit events
   - Add SINK event subscription

2. **UI module:**
   - Create the module structure in `modules/audio/src/ui/`
   - Implement `AudioWidget` (bar icon)
   - Implement `AudioPopup` (control popup)
   - Implement `build_ui`

3. **Integration:**
   - Add dependencies to `modules/audio/Cargo.toml`
   - Integrate into `bar/src/app.rs`
   - Add CSS styles

4. **Testing and documentation:**
   - Test all functions
   - Update documentation

## Notes

- All backend interaction logic lives in the UI module (`modules/audio/src/ui/`)
- The UI module uses `Client` from the backend to send commands and receive events
- Use `MainContext::default().invoke()` to update UI from the PulseAudio thread
- The popup can be implemented as a `Popover` (anchored to the icon) or a separate window
- A `Popover` is recommended for a more native feel
- The applications list should update dynamically when apps are added or removed
