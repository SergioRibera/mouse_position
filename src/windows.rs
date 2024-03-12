//! Implementation for Windows.

use winapi::{
    shared::windef::POINT,
    um::winuser::{GetCursorPos, GetPhysicalCursorPos},
};

use crate::{error::MousePosition, MouseExt};

#[derive(Default, Clone)]
pub struct WinMouse;

unsafe impl Sync for WinMouse {}
unsafe impl Send for WinMouse {}

impl MouseExt for WinMouse {
    fn get_pos(&mut self) -> Result<(i32, i32), MousePosition> {
        let mut point = POINT { x: 0, y: 0 };
        let result = unsafe { GetCursorPos(&mut point) };
        match result {
            1 => Ok((point.x, point.y)),
            _ => Err(MousePosition::NoMouseFound),
        }
    }

    fn get_physical_pos(&self) -> Result<(i32, i32), MousePosition> {
        let mut point = POINT { x: 0, y: 0 };
        let result = unsafe { GetPhysicalCursorPos(&mut point) };
        match result {
            1 => Ok((point.x, point.y)),
            _ => Err(MousePosition::NoMouseFound),
        }
    }
}
