mod protocol;

use std::os::unix::net::UnixStream;

use crate::protocol::{display, types::WlNewId};

fn connect_to_wayland_socket() -> anyhow::Result<UnixStream> {
    let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;
    let wayland_display = std::env::var("WAYLAND_DISPLAY")?;

    let socket_path = format!("{xdg_runtime_dir}/{wayland_display}");

    let stream = UnixStream::connect(socket_path)?;

    Ok(stream)
}

fn main() -> anyhow::Result<()> {
    let mut stream = connect_to_wayland_socket()?;
    display::request::get_registry(&mut stream, WlNewId(1))?;

    Ok(())
}
