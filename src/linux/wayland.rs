use std::sync::Arc;

use sctk::compositor::{CompositorHandler, CompositorState};
use sctk::globals::GlobalData;
use sctk::output::{OutputHandler, OutputState};
use sctk::reexports::calloop::EventLoop;
use sctk::reexports::protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::WpCursorShapeDeviceV1;
use sctk::reexports::protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use sctk::registry::{ProvidesRegistryState, RegistryState};
use sctk::seat::pointer::{PointerData, PointerEventKind, PointerHandler};
use sctk::seat::{SeatData, SeatState};
use sctk::subcompositor::SubcompositorState;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_pointer::WlPointer;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{delegate_dispatch, Connection, Dispatch, EventQueue, QueueHandle};

use smithay_client_toolkit as sctk;

use crate::MouseExt;

fn logical_to_physical_rounded(size: Option<(i32, i32)>, scale_factor: f64) -> Option<(i32, i32)> {
    size.map(|(w, h)| {
        let width = w as f64 * scale_factor;
        let height = h as f64 * scale_factor;
        (width.round() as i32, height.round() as i32)
    })
}

// #[derive(Clone)]
// pub struct WaylandMouse(WM);

impl Default for WaylandMouse {
    fn default() -> Self {
        // Inicializa el entorno de Wayland
        let conn = Connection::connect_to_env().unwrap();
        let display = conn.display();

        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh: QueueHandle<WaylandMouse> = event_queue.handle();

        // Self(WM::new().expect("No Wayland WM detected"))

        let registry_state = RegistryState::new(&globals);
        let compositor_state = CompositorState::bind(&globals, &qh).unwrap();
        let subcompositor_state =
            match SubcompositorState::bind(compositor_state.wl_compositor().clone(), &globals, &qh)
            {
                Ok(c) => Some(c),
                Err(e) => {
                    println!("Subcompositor protocol not available, ignoring CSD: {e:?}");
                    None
                }
            };

        let seat_state = SeatState::new(&globals, &qh);

        Self {
            registry_state,
            event_queue,
            output_state: OutputState::new(&globals, &qh),
            compositor_state: Arc::new(compositor_state),
            subcompositor_state: subcompositor_state.map(Arc::new),
            seat_state,

            // relative_pointer: Some(RelativePointerState::bind(&globals, &qh)),

            // Make it true by default.
            dispatched_events: true,
            scale_factor: 1,
            position: None,
        }
    }
}

unsafe impl Sync for WaylandMouse {}
unsafe impl Send for WaylandMouse {}

impl MouseExt for WaylandMouse {
    fn get_pos(&mut self) -> Result<(i32, i32), crate::error::MousePosition> {
        // self.0.get_pos()
        // let _ = self
        //     .event_queue
        //     .blocking_dispatch(&mut ())
        //     .map_err(|_| crate::error::MousePosition::WMNotDetected)?;
        self.position.ok_or(crate::error::MousePosition::BadExtract)
    }

    fn get_physical_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        logical_to_physical_rounded(self.position, self.scale_factor as f64)
            .ok_or(crate::error::MousePosition::BadExtract)
    }
}

pub struct WaylandMouse {
    /// The WlRegistry.
    pub registry_state: RegistryState,

    pub event_queue: EventQueue<Self>,

    /// The state of the WlOutput handling.
    pub output_state: OutputState,

    /// The compositor state which is used to create new windows and regions.
    pub compositor_state: Arc<CompositorState>,

    /// The state of the subcompositor.
    pub subcompositor_state: Option<Arc<SubcompositorState>>,

    /// The seat state responsible for all sorts of input.
    pub seat_state: SeatState,

    /// Relative pointer.
    // pub relative_pointer: Option<RelativePointerState>,

    /// Whether we have dispatched events to the user thus we want to
    /// send `AboutToWait` and normally wakeup the user.
    pub dispatched_events: bool,

    scale_factor: i32,
    position: Option<(i32, i32)>,
}

impl CompositorHandler for WaylandMouse {
    fn scale_factor_changed(
        &mut self,
        conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        surface: &wayland_client::protocol::wl_surface::WlSurface,
        new_factor: i32,
    ) {
        self.scale_factor = new_factor;
        println!("New Scale Factor: {new_factor}");
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &wayland_client::QueueHandle<Self>,
        _surface: &wayland_client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
    }
}

impl PointerHandler for WaylandMouse {
    fn pointer_frame(
        &mut self,
        conn: &Connection,
        qh: &wayland_client::QueueHandle<Self>,
        pointer: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[sctk::seat::pointer::PointerEvent],
    ) {
        // let seat = pointer
        //     .data::<PointerData>()
        //     .expect("failed to get pointer data.")
        //     .seat();
        // let seat_state = self.seats.get(&seat.id()).unwrap();

        for event in events {
            // let surface = &event.surface;

            // // The parent surface.
            // let parent_surface = match event.surface.data::<SurfaceData>() {
            //     Some(data) => data.parent_surface().unwrap_or(surface),
            //     None => continue,
            // };

            println!("Event: {event:?}");
            match event.kind {
                PointerEventKind::Motion { .. }
                | PointerEventKind::Enter { .. }
                | PointerEventKind::Leave { .. } => {
                    self.position = Some((event.position.0 as i32, event.position.1 as i32));
                }
                _ => self.position = None,
            }
        }
    }
}

impl OutputHandler for WaylandMouse {
    fn output_state(&mut self) -> &mut sctk::output::OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wayland_client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl ProvidesRegistryState for WaylandMouse {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    sctk::registry_handlers![OutputState];
}

impl Dispatch<WlRegistry, GlobalData> for WaylandMouse {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        data: &GlobalData,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSeat, SeatData> for WaylandMouse {
    fn event(
        state: &mut Self,
        proxy: &WlSeat,
        event: <WlSeat as wayland_client::Proxy>::Event,
        data: &SeatData,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WpCursorShapeDeviceV1, GlobalData, WaylandMouse> for SeatState {
    fn event(
        _state: &mut WaylandMouse,
        _proxy: &WpCursorShapeDeviceV1,
        _event: <WpCursorShapeDeviceV1 as wayland_client::Proxy>::Event,
        _data: &GlobalData,
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<WaylandMouse>,
    ) {
        unreachable!("wp_cursor_shape_manager has no events")
    }
}

impl Dispatch<WpCursorShapeManagerV1, GlobalData, WaylandMouse> for SeatState {
    fn event(
        _state: &mut WaylandMouse,
        _proxy: &WpCursorShapeManagerV1,
        _event: <WpCursorShapeManagerV1 as wayland_client::Proxy>::Event,
        _data: &GlobalData,
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<WaylandMouse>,
    ) {
        unreachable!("wp_cursor_device_manager has no events")
    }
}

sctk::delegate_subcompositor!(WaylandMouse);
sctk::delegate_compositor!(WaylandMouse);
sctk::delegate_output!(WaylandMouse);
sctk::delegate_registry!(WaylandMouse);
delegate_dispatch!(WaylandMouse: [ WlPointer: PointerData] => SeatState);
delegate_dispatch!(WaylandMouse: [ WpCursorShapeManagerV1: GlobalData] => SeatState);
delegate_dispatch!(WaylandMouse: [ WpCursorShapeDeviceV1: GlobalData] => SeatState);
// delegate_dispatch!(WaylandMouse: [ZwpPointerConstraintsV1: GlobalData] => PointerConstraintsState);
// delegate_dispatch!(WaylandMouse: [ZwpLockedPointerV1: GlobalData] => PointerConstraintsState);
// delegate_dispatch!(WaylandMouse: [ZwpConfinedPointerV1: GlobalData] => PointerConstraintsState);
