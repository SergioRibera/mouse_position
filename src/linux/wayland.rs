use std::os::fd::AsFd;

use smithay_client_toolkit::reexports::protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use smithay_client_toolkit::reexports::protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1::ZxdgOutputManagerV1;
use smithay_client_toolkit::reexports::protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1};
use smithay_client_toolkit::reexports::protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{self, Anchor};
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_shm::{Format, WlShm};
use wayland_client::{Connection, EventQueue};

mod dispatch;
mod state;

use crate::MouseExt;

use state::State;

pub struct WaylandMouse {
    event_queue: EventQueue<State>,
    state: State,
}

unsafe impl Sync for WaylandMouse {}
unsafe impl Send for WaylandMouse {}

impl Default for WaylandMouse {
    fn default() -> Self {
        let connection = Connection::connect_to_env().unwrap();
        let (globals, _) = registry_queue_init::<State>(&connection).unwrap();

        let mut event_queue = connection.new_event_queue::<State>();
        let qh = event_queue.handle();

        let mut state = State::default();

        let wmcompositer = globals.bind::<WlCompositor, _, _>(&qh, 1..=5, ()).unwrap();

        let cursor_manager = globals
            .bind::<WpCursorShapeManagerV1, _, _>(&qh, 1..=1, ())
            .ok();

        let shm = globals.bind::<WlShm, _, _>(&qh, 1..=1, ()).unwrap();

        state.cursor_manager = cursor_manager;

        globals.bind::<WlSeat, _, _>(&qh, 1..=1, ()).unwrap();

        let _ = connection.display().get_registry(&qh, ()); // so if you want WlOutput, you need to
                                                            // register this

        event_queue.blocking_dispatch(&mut state).unwrap();

        let xdg_output_manager = globals
            .bind::<ZxdgOutputManagerV1, _, _>(&qh, 1..=3, ())
            .unwrap();

        for wloutput in state.outputs.iter() {
            let zwloutput = xdg_output_manager.get_xdg_output(wloutput.get_output(), &qh, ());
            state
                .zxdg_outputs
                .push(state::ZXdgOutputInfo::new(zwloutput));
        }

        event_queue.blocking_dispatch(&mut state).unwrap();

        // you will find you get the outputs, but if you do not
        // do the step before, you get empty list

        let layer_shell = globals
            .bind::<ZwlrLayerShellV1, _, _>(&qh, 3..=4, ())
            .unwrap();
        let region = wmcompositer.create_region(&qh, ());
        region.add(0, 0, 0, 0);

        // so it is the same way, to get surface detach to protocol, first get the shell, like wmbase
        // or layer_shell or session-shell, then get `surface` from the wl_surface you get before, and
        // set it
        // finally thing to remember is to commit the surface, make the shell to init.
        for (index, (wloutput, zwlinfo)) in state
            .outputs
            .iter()
            .zip(state.zxdg_outputs.iter())
            .enumerate()
        {
            let wl_surface = wmcompositer.create_surface(&qh, ());
            let (init_w, init_h) = (zwlinfo.width, zwlinfo.height);
            // this example is ok for both xdg_surface and layer_shell

            let layer = layer_shell.get_layer_surface(
                &wl_surface,
                Some(wloutput.get_output()),
                Layer::Overlay,
                format!("__nobody_mouse_position_{index}"),
                &qh,
                (),
            );
            layer.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right | Anchor::Bottom);
            layer.set_exclusive_zone(-1);
            layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
            // TODO: check why
            // wl_surface.set_input_region(Some(&region));
            layer.set_size(init_w as u32, init_h as u32);

            wl_surface.commit(); // so during the init Configure of the shell, a buffer, atleast a buffer is needed.
                                 // and if you need to reconfigure it, you need to commit the wl_surface again
                                 // so because this is just an example, so we just commit it once
                                 // like if you want to reset anchor or KeyboardInteractivity or resize, commit is needed

            let file = tempfile::tempfile().unwrap();
            file.set_len((init_w * init_h * 4) as u64).unwrap();
            let pool = shm.create_pool(file.as_fd(), init_w * init_h * 4, &qh, ());
            let buffer =
                pool.create_buffer(0, init_w, init_h, init_w * 4, Format::Argb8888, &qh, ());

            state.wl_surfaces.push(state::LayerSurfaceInfo {
                layer,
                wl_surface,
                buffer,
            });
        }

        event_queue.blocking_dispatch(&mut state).unwrap();

        Self { event_queue, state }
    }
}

impl MouseExt for WaylandMouse {
    fn get_pos(&mut self) -> Result<(i32, i32), crate::error::MousePosition> {
        _ = self.event_queue.blocking_dispatch(&mut self.state).unwrap();
        Ok(self.state.current_pos)
    }

    fn get_physical_pos(&mut self) -> Result<(i32, i32), crate::error::MousePosition> {
        Err(crate::error::MousePosition::BadExtract)
    }
}
