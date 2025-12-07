use std::os::fd::{AsRawFd, BorrowedFd};

use memmap2::MmapMut;
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_output::{Event as WlOutputEvent, WlOutput},
        wl_registry,
        wl_shm::{Format, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{Event as LayerSurfaceEvent, ZwlrLayerSurfaceV1},
};

const INTERFACE_COMPOSITOR: &str = "wl_compositor";
const INTERFACE_SHM: &str = "wl_shm";
const INTERFACE_LAYER_SHELL: &str = "zwlr_layer_shell_v1";
const INTERFACE_OUTPUT: &str = "wl_output";

// This struct represents the state of our app. This simple app does not
// need any state, but this type still supports the `Dispatch` implementations.
pub struct AppData {
    pub compositor: Option<WlCompositor>,
    pub shm: Option<WlShm>,
    pub layer_shell: Option<ZwlrLayerShellV1>,
    pub output: Option<WlOutput>,

    pub surface: Option<WlSurface>,
    pub layer_surface: Option<ZwlrLayerSurfaceV1>,

    pub buffers: Vec<WlBuffer>,

    pub running: bool,
}

impl AppData {
    pub fn new() -> Self {
        AppData {
            compositor: None,
            shm: None,
            layer_shell: None,
            output: None,
            surface: None,
            layer_surface: None,
            buffers: Vec::new(),
            running: true,
        }
    }
}

impl Dispatch<WlCompositor, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: wayland_client::protocol::wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // wl_compositor обычно не шлёт событий, так что здесь пусто
    }
}

impl Dispatch<WlShm, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlShm,
        _event: wayland_client::protocol::wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // wl_shm тоже почти без событий, можно игнорировать
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrLayerShellV1,
        _event: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // На уровне самого глобала layer_shell событий нет, все интересные события
        // идут от ZwlrLayerSurfaceV1 (который мы пока не создали).
    }
}
impl Dispatch<WlOutput, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlOutput,
        _event: WlOutputEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<WlShmPool, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlShmPool,
        _event: wayland_client::protocol::wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // wl_shm_pool обычно не шлёт событий
    }
}

impl Dispatch<WlBuffer, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlBuffer,
        _event: wayland_client::protocol::wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // wl_shm_pool обычно не шлёт событий
    }
}

impl Dispatch<WlSurface, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: wayland_client::protocol::wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // wl_shm_pool обычно не шлёт событий
    }
}

// Implement `Dispatch<WlRegistry, ()> for our state. This provides the logic
// to be able to process events for the wl_registry interface.
//
// The second type parameter is the user-data of our implementation. It is a
// mechanism that allows you to associate a value to each particular Wayland
// object, and allow different dispatching logic depending on the type of the
// associated value.
//
// In this example, we just use () as we don't have any value to associate. See
// the `Dispatch` documentation for more details about this.
impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        queue_handle: &QueueHandle<AppData>,
    ) {
        // When receiving events from the wl_registry, we are only interested in the
        // `global` event, which signals a new available global.
        // When receiving this event, we just print its characteristics in this example.
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} (v{})", name, interface, version);
            match interface.as_str() {
                INTERFACE_COMPOSITOR => {
                    // Here we could bind the wl_compositor global if we wanted to
                    // use it later in our program.
                    let compositor =
                        registry.bind::<WlCompositor, _, _>(name, version, queue_handle, ());
                    state.compositor = Some(compositor);
                }
                INTERFACE_SHM => {
                    // Here we could bind the wl_shm global if we wanted to
                    // use it later in our program.
                    let wl_shm = registry.bind::<WlShm, _, _>(name, version, queue_handle, ());
                    state.shm = Some(wl_shm);
                }
                INTERFACE_LAYER_SHELL => {
                    // Here we could bind the zwlr_layer_shell_v1 global if we wanted to
                    // use it later in our program.
                    let interface_layer_shell =
                        registry.bind::<ZwlrLayerShellV1, _, _>(name, version, queue_handle, ());
                    state.layer_shell = Some(interface_layer_shell);
                }
                INTERFACE_OUTPUT => {
                    // Here we could bind the zwlr_layer_shell_v1 global if we wanted to
                    // use it later in our program.
                    let interface_output =
                        registry.bind::<WlOutput, _, _>(name, version, queue_handle, ());
                    state.output = Some(interface_output);
                }
                _ => {}
            }
        }
    }
}

/// Dispatch implementation for ZwlrLayerSurfaceV1 events.
impl Dispatch<ZwlrLayerSurfaceV1, ()> for AppData {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: LayerSurfaceEvent,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            LayerSurfaceEvent::Configure {
                serial,
                width,
                height,
            } => {
                println!("configure: serial={serial}, width={width}, height={height}");

                // обязаны ack-нуть configure
                layer_surface.ack_configure(serial);

                let width = if width == 0 { 1920 } else { width }; // fallback
                let height = if height == 0 { 30 } else { height };

                draw_solid_color(state, width, height, 0xFF_33_66_99, qh);
            }
            LayerSurfaceEvent::Closed => {
                println!("layer-surface closed, выходим");
                state.running = false;
            }
            _ => {}
        }
    }
}

fn draw_solid_color(
    state: &mut AppData,
    width: u32,
    height: u32,
    argb: u32,
    qh: &QueueHandle<AppData>,
) {
    // extract options values
    let shm = state.shm.as_ref().expect("no wl_shm");
    let surface = state.surface.as_ref().expect("no wl_surface");

    let stride = (width * 4) as i32;
    let size = stride as usize * height as usize;

    let file = tempfile::tempfile().expect("failed to create tmpfile");
    file.set_len(size as u64).expect("failed to set_len");
    let borrowed_fd = unsafe { BorrowedFd::borrow_raw(file.as_raw_fd()) };

    // mmap
    let mut mmap = unsafe { MmapMut::map_mut(&file).expect("mmap failed") };

    // fill with color
    let pixels = unsafe { std::slice::from_raw_parts_mut(mmap.as_mut_ptr() as *mut u32, size / 4) };
    for p in pixels {
        *p = argb;
    }

    let pool = shm.create_pool(borrowed_fd, size as i32, qh, ());
    let buffer = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        stride,
        Format::Argb8888,
        qh,
        (),
    );

    surface.attach(Some(&buffer), 0, 0);
    surface.damage_buffer(0, 0, width as i32, height as i32);
    surface.commit();

    state.buffers.push(buffer);
}
