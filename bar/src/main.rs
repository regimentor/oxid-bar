mod app_data;
mod desktop_file;
mod hyprland_api;
mod icon_fetcher;
use wayland_client::{protocol::wl_surface::WlSurface, Connection};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer,
    zwlr_layer_surface_v1::{Anchor, ZwlrLayerSurfaceV1},
};

use crate::{app_data::AppData, hyprland_api::HyprWorkspaces};

// The main function of our program
fn main() {
    // Create a Wayland connection by connecting to the server through the
    // environment-provided configuration.
    let conn = Connection::connect_to_env().unwrap_or_else(|e| {
        panic!(
            "Failed to connect to Wayland server: {}, check envs WAYLAND_DISPLAY, WAYLAND_SOCKET",
            e
        );
    });

    // Retrieve the WlDisplay Wayland object from the connection. This object is
    // the starting point of any Wayland program, from which all other objects will
    // be created.
    let display = conn.display();

    // Create an event queue for our event processing
    let mut event_queue = conn.new_event_queue();
    // And get its handle to associate new objects to it
    let qh = event_queue.handle();

    // Create a wl_registry object by sending the wl_display.get_registry request.
    // This method takes two arguments: a handle to the queue that the newly created
    // wl_registry will be assigned to, and the user-data that should be associated
    // with this registry (here it is () as we don't need user-data).
    let _registry = display.get_registry(&qh, ());

    // At this point everything is ready, and we just need to wait to receive the events
    // from the wl_registry. Our callback will print the advertised globals.
    println!("Advertised globals:");

    // To actually receive the events, we invoke the `roundtrip` method. This method
    // is special and you will generally only invoke it during the setup of your program:
    // it will block until the server has received and processed all the messages you've
    // sent up to now.
    //
    // In our case, that means it'll block until the server has received our
    // wl_display.get_registry request, and as a reaction has sent us a batch of
    // wl_registry.global events.
    //
    // `roundtrip` will then empty the internal buffer of the queue it has been invoked
    // on, and thus invoke our `Dispatch` implementation that prints the list of advertised
    // globals.

    let mut state = AppData::new();

    event_queue.roundtrip(&mut state).unwrap();

    // check that we have the globals we need
    let compositor = state.compositor.as_ref().expect("wl_compositor not found");
    let layer_shell = state
        .layer_shell
        .as_ref()
        .expect("zwlr_layer_shell_v1 not found");
    let _shm = state.shm.as_ref().expect("wl_shm not found");
    let output = state.output.as_ref();

    // create a surface
    let surface: WlSurface = compositor.create_surface(&qh, ());
    state.surface = Some(surface.clone());

    // layer-surface, create a top panel
    let height: u32 = 30;
    let layer_surface: ZwlrLayerSurfaceV1 = layer_shell.get_layer_surface(
        &surface,
        output,                   // Some(&output) – только на одном мониторе
        Layer::Top,               // слой: сверху
        "rust-waybar-min".into(), // namespace
        &qh,
        (),
    );
    layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    layer_surface.set_size(0, height);
    layer_surface.set_exclusive_zone(height as i32);
    state.layer_surface = Some(layer_surface);

    // commit state to get the first configure event
    surface.commit();

    // Get Hyprland workspaces and clients
    let hyprland_api_workspaces =
        HyprWorkspaces::init().expect("error while get Hyprland workspaces");

    for (id, ws) in hyprland_api_workspaces.map.iter() {
        println!("Workspace {}: {}", id, ws);
        for client in ws.clients.iter() {
            println!("    Client: {}", client);
            println!(
                "        Desktop File: {}",
                if let Some(df) = &client.desktop_file {
                    format!("{}", df)
                } else {
                    String::from("Not Found")
                }
            );
            for icon in client.icons.iter() {
                println!("        Icon path: {}", icon);
            }
        }
    }

    // Enter the event loop
    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .expect("Erro while event haldling (loop)");
    }
}
