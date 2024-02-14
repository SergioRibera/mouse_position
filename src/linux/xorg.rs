use x11_dl::xlib::{Display, Window, Xlib};

use crate::error::MousePosition as MouseError;
use crate::MouseExt;

pub struct XorgMouse {
    xlib: Xlib,
    display: *mut Display,
    screens: Vec<u64>,
}

unsafe impl Sync for XorgMouse {}
unsafe impl Send for XorgMouse {}

impl Default for XorgMouse {
    fn default() -> Self {
        let xlib = Xlib::open().expect("An error occurred while opening the X11 session");
        let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };

        let screens_n = unsafe { (xlib.XScreenCount)(display) };
        let mut screens = Vec::new();

        for s in 0..screens_n {
            screens.push(unsafe { (xlib.XRootWindow)(display, s) });
        }

        Self {
            xlib,
            display,
            screens,
        }
    }
}

impl MouseExt for XorgMouse {
    fn get_pos(&mut self) -> Result<(i32, i32), MouseError> {
        // Get the pointer position
        let mut root: Window = 0;
        let mut child: Window = 0;
        let mut x: i32 = -1;
        let mut y: i32 = -1;
        let mut win_x: i32 = 0;
        let mut win_y: i32 = 0;
        let mut mask: u32 = 0;

        let mut res = -1;
        for screen in &self.screens {
            res = unsafe {
                (self.xlib.XQueryPointer)(
                    self.display,
                    *screen,
                    &mut root,
                    &mut child,
                    &mut x,
                    &mut y,
                    &mut win_x,
                    &mut win_y,
                    &mut mask,
                )
            };
            if res == 1 {
                break;
            }
        }

        if res != 1 {
            return Err(MouseError::NoMouseFound);
        }

        if x == -1 || y == -1 {
            return Err(MouseError::BadExtract);
        }

        Ok((x, y))
    }

    fn get_physical_pos(&self) -> Result<(i32, i32), MouseError> {
        Err(MouseError::Unimplemented)
    }
}

impl Drop for XorgMouse {
    fn drop(&mut self) {
        unsafe {
            (self.xlib.XCloseDisplay)(self.display);
        }
    }
}
