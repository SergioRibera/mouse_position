use super::state::{self, LayerSurfaceInfo, State};
use smithay_client_toolkit::reexports::{
    protocols::{
        wp::cursor_shape::v1::client::{
            wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
            wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        },
        xdg::{
            shell::client::{xdg_toplevel::XdgToplevel, xdg_wm_base},
            xdg_output::zv1::client::{
                zxdg_output_manager_v1::ZxdgOutputManagerV1, zxdg_output_v1,
            },
        },
    },
    protocols_wlr::layer_shell::v1::client::{
        zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1,
    },
};
use wayland_client::{
    delegate_noop,
    globals::GlobalListContents,
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_output, wl_pointer,
        wl_region::WlRegion,
        wl_registry,
        wl_seat::{self},
        wl_shm::WlShm,
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for State {
    fn event(
        state: &mut Self,
        surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
            surface.ack_configure(serial);
            let Some(LayerSurfaceInfo {
                wl_surface, buffer, ..
            }) = state.wl_surfaces.iter().find(|info| info.layer == *surface)
            else {
                return;
            };
            wl_surface.attach(Some(buffer), 0, 0);
            wl_surface.commit();
        }
    }
}

impl Dispatch<zxdg_output_v1::ZxdgOutputV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &zxdg_output_v1::ZxdgOutputV1,
        event: <zxdg_output_v1::ZxdgOutputV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let Some(info) = state
            .zxdg_outputs
            .iter_mut()
            .find(|info| info.zxdg_output == *proxy)
        else {
            return;
        };
        match event {
            zxdg_output_v1::Event::LogicalSize { width, height } => {
                info.height = height;
                info.width = width;
            }
            zxdg_output_v1::Event::LogicalPosition { x, y } => {
                info.start_x = x;
                info.start_y = y;
            }
            zxdg_output_v1::Event::Name { name } => info.name = name,
            zxdg_output_v1::Event::Description { description } => info.description = description,
            _ => {}
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for state::State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for state::State {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };

        if interface == wl_output::WlOutput::interface().name {
            let output = proxy.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
            state.outputs.push(state::WlOutputInfo::new(output));
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for state::State {
    fn event(
        state: &mut Self,
        wl_output: &wl_output::WlOutput,
        event: <wl_output::WlOutput as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let output = state
            .outputs
            .iter_mut()
            .find(|x| x.get_output() == wl_output)
            .unwrap();

        match event {
            wl_output::Event::Name { name } => {
                output.name = name;
            }
            wl_output::Event::Description { description } => {
                output.description = description;
            }
            wl_output::Event::Mode { width, height, .. } => {
                output.size = (width, height);
            }
            _ => (),
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for state::State {
    fn event(
        _state: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: <xdg_wm_base::XdgWmBase as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for state::State {
    fn event(
        _state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: <wl_seat::WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Pointer) {
                seat.get_pointer(qh, ());
            }
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for state::State {
    fn event(
        dispatch_state: &mut Self,
        _pointer: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface,
                surface_x,
                surface_y,
                ..
            } => {
                let Some(LayerSurfaceInfo { wl_surface, .. }) = dispatch_state
                    .wl_surfaces
                    .iter()
                    .find(|info| info.wl_surface == surface)
                else {
                    return;
                };
                let current_screen = dispatch_state
                    .wl_surfaces
                    .iter()
                    .position(|info| info.wl_surface == surface)
                    .unwrap();
                dispatch_state.current_screen = current_screen;
                let start_x = dispatch_state.zxdg_outputs[dispatch_state.current_screen].start_x;
                let start_y = dispatch_state.zxdg_outputs[dispatch_state.current_screen].start_y;

                dispatch_state.current_pos =
                    (surface_x as i32 + start_x, surface_y as i32 + start_y);
                wl_surface.commit();
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                let start_x = dispatch_state.zxdg_outputs[dispatch_state.current_screen].start_x;
                let start_y = dispatch_state.zxdg_outputs[dispatch_state.current_screen].start_y;

                dispatch_state.current_pos =
                    (surface_x as i32 + start_x, surface_y as i32 + start_y);
            }
            _ => {}
        }
    }
}

impl Dispatch<WlRegion, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegion,
        _event: <WlRegion as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(State: ignore WlCompositor); // WlCompositor is need to create a surface
delegate_noop!(State: ignore WlSurface); // surface is the base needed to show buffer

delegate_noop!(State: ignore WlShm); // shm is used to create buffer pool
delegate_noop!(State: ignore XdgToplevel); // so it is the same with layer_shell, private a
                                           // place for surface
delegate_noop!(State: ignore WlShmPool); // so it is pool, created by wl_shm
delegate_noop!(State: ignore WlBuffer); // buffer show the picture
delegate_noop!(State: ignore ZwlrLayerShellV1); // it is similar with xdg_toplevel, also the
                                                // ext-session-shell
delegate_noop!(State: ignore ZxdgOutputManagerV1);

delegate_noop!(State: ignore WpCursorShapeManagerV1);
delegate_noop!(State: ignore WpCursorShapeDeviceV1);
