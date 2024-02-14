//! Implementation for Linux.

use std::env::var_os;

use crate::MouseExt;

mod wayland;
mod xorg;

pub struct LinuxMouse {
    inner: Box<dyn MouseExt>,
}

unsafe impl Sync for LinuxMouse {}
unsafe impl Send for LinuxMouse {}

fn is_wayland() -> bool {
    var_os("WAYLAND_DISPLAY")
        .or(var_os("XDG_SESSION_TYPE"))
        .is_some_and(|v| {
            v.to_str()
                .unwrap_or_default()
                .to_lowercase()
                .contains("wayland")
        })
}

impl Default for LinuxMouse {
    fn default() -> Self {
        Self {
            inner: if is_wayland() {
                Box::<wayland::WaylandMouse>::default()
            } else {
                Box::<xorg::XorgMouse>::default()
            },
        }
    }
}

impl MouseExt for LinuxMouse {
    fn get_pos(&mut self) -> Result<(i32, i32), crate::error::MousePosition> {
        self.inner.get_pos()
    }

    fn get_physical_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        self.inner.get_physical_pos()
    }
}
