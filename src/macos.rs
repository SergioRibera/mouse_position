//! Implementation for MacOS.

use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use crate::MouseExt;

#[derive(Default, Clone)]
pub struct MacMouse;

unsafe impl Sync for MacMouse {}
unsafe impl Send for MacMouse {}

impl MouseExt for MacMouse {
    fn get_pos(&mut self) -> Result<(i32, i32), crate::error::MousePosition> {
        let event =
            CGEvent::new(CGEventSource::new(CGEventSourceStateID::CombinedSessionState).unwrap());
        match event {
            Ok(event) => {
                let point = event.map_err(|_| MouseError::NoMouseFound)?.location();
                Ok((point.x as i32, point.y as i32))
            }
            Err(_) => return Err(crate::error::MouseError::BadExtract),
        }
    }

    fn get_physical_pos(&mut self) -> Result<(i32, i32), crate::error::MousePosition> {
        Err(crate::error::MousePosition::Unimplemented)
    }
}
