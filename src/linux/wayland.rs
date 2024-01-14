use crate::MouseExt;

#[derive(Default, Clone)]
pub struct WaylandMouse;

unsafe impl Sync for WaylandMouse {}
unsafe impl Send for WaylandMouse {}

impl MouseExt for WaylandMouse {
    fn get_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        Ok((0, 0))
    }

    fn get_physical_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        Ok((0, 0))
    }
}
