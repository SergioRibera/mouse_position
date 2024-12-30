use smithay_client_toolkit::reexports::protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use smithay_client_toolkit::reexports::protocols::xdg::xdg_output::zv1::client::zxdg_output_v1;
use smithay_client_toolkit::reexports::protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::{wl_output::WlOutput, wl_surface::WlSurface};

#[derive(Debug)]
pub struct ZXdgOutputInfo {
    pub zxdg_output: zxdg_output_v1::ZxdgOutputV1,
    pub width: i32,
    pub height: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub name: String,
    pub description: String,
}

impl ZXdgOutputInfo {
    pub fn new(zxdgoutput: zxdg_output_v1::ZxdgOutputV1) -> Self {
        Self {
            zxdg_output: zxdgoutput,
            width: 0,
            height: 0,
            start_x: 0,
            start_y: 0,
            name: "".to_string(),
            description: "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WlOutputInfo {
    pub output: WlOutput,
    pub description: String,
    pub name: String,
    pub size: (i32, i32),
}

impl WlOutputInfo {
    pub fn new(output: WlOutput) -> Self {
        Self {
            output,
            description: "".to_string(),
            name: "".to_string(),
            size: (0, 0),
        }
    }

    pub fn get_output(&self) -> &WlOutput {
        &self.output
    }
}

#[derive(Debug)]
pub struct State {
    pub outputs: Vec<WlOutputInfo>,
    pub zxdg_outputs: Vec<ZXdgOutputInfo>,
    pub wl_surfaces: Vec<LayerSurfaceInfo>,
    pub current_pos: (i32, i32),
    pub current_screen: usize,
    pub cursor_manager: Option<WpCursorShapeManagerV1>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            outputs: Vec::new(),
            zxdg_outputs: Vec::new(),
            wl_surfaces: Vec::new(),
            current_pos: (0, 0),
            current_screen: 0,
            cursor_manager: None,
        }
    }
}

#[derive(Debug)]
pub struct LayerSurfaceInfo {
    pub layer: ZwlrLayerSurfaceV1,
    pub wl_surface: WlSurface,
    pub buffer: WlBuffer,
}
