//! Implementation for Windows.

use winapi::{
    shared::windef::POINT,
    um::winuser::{GetCursorPos, GetPhysicalCursorPos},
};

use crate::MouseExt;

#[derive(Default, Clone)]
pub struct WinMouse;

unsafe impl Sync for WinMouse {}
unsafe impl Send for WinMouse {}

impl MouseExt for LinuxMouse {
    fn get_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        let mut point = POINT { x: 0, y: 0 };
        let result = unsafe { GetCursorPos(&mut point) };

        if result == 1 {
            return Ok(point.x, point.y);
        }
    }

    fn get_physical_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        let mut point = POINT { x: 0, y: 0 };
        let result = unsafe { GetPhysicalCursorPos(&mut point) };

        if result == 1 {
            return Ok(point.x, point.y);
        }
    }
}
