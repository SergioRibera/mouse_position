//! A a simple crate to get the mouse position in a cross platform way.
//! It uses winapi crate to get the mouse position on windows, x11-dl for linux, and core-graphics for macos.
//! Example Usage:
//! ```rust no_compile
//! use mouse_position::{Mouse, MouseExt};
//!
//! fn main() {
//!     let mut mouse = Mouse::default();
//!
//!     match mouse.get_pos() {
//!         Ok((x, y)) => println!("x: {x}, y: {y}"),
//!         Err(e) => println!("{e:?}"),
//!     }
//! }
//! ```

pub mod error;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use error::MousePosition as MouseError;

#[cfg(target_os = "linux")]
pub use linux::LinuxMouse as Mouse;
#[cfg(target_os = "macos")]
pub use macos::MacMouse as Mouse;
#[cfg(target_os = "windows")]
pub use windows::WinMouse as Mouse;

pub trait MouseExt {
    fn get_pos(&mut self) -> Result<(i32, i32), MouseError>;
    fn get_physical_pos(&mut self) -> Result<(i32, i32), MouseError>;
}
