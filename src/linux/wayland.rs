use crate::MouseExt;

fn write_to_socket(path: String, content: &str) -> Result<String, crate::error::MousePosition> {
    use std::io::prelude::*;
    use std::os::unix::net::UnixStream;
    let mut stream = UnixStream::connect(path)?;

    stream.write_all(&content.as_bytes())?;

    let mut response = vec![];

    const BUF_SIZE: usize = 8192;
    let mut buf = [0; BUF_SIZE];
    loop {
        let num_read = stream.read(&mut buf)?;
        let buf = &buf[..num_read];
        response.append(&mut buf.to_vec());
        if num_read == 0 || num_read != BUF_SIZE {
            break;
        }
    }

    Ok(String::from_utf8(response)?)
}

#[derive(Clone)]
enum WM {
    Hyprland,
    #[allow(unused)]
    KDE,
}

impl WM {
    pub fn new() -> Result<Self, crate::error::MousePosition> {
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map(|_| WM::Hyprland)
            .map_err(|_| crate::error::MousePosition::WMNotDetected)
    }

    pub fn socket(&self) -> Result<String, crate::error::MousePosition> {
        match self {
            WM::Hyprland => std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
                .map_err(|_| crate::error::MousePosition::SocketNotFound)
                .map(|sig| format!("/tmp/hypr/{sig}/.socket.sock")),
            _ => Err(crate::error::MousePosition::SocketNotFound),
        }
    }

    pub fn get_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        match self {
            WM::Hyprland => {
                let raw = write_to_socket(self.socket()?, "/cursorpos")?;
                let pos = raw
                    .split_once(", ")
                    .map(|(x, y)| {
                        (
                            x.parse::<i32>().expect("Cannot parse X"),
                            y.parse::<i32>().expect("Cannot parse Y"),
                        )
                    })
                    .ok_or(crate::error::MousePosition::BadExtract)?;
                Ok(pos)
            }
            _ => Err(crate::error::MousePosition::Unimplemented),
        }
    }
}

#[derive(Clone)]
pub struct WaylandMouse(WM);

impl Default for WaylandMouse {
    fn default() -> Self {
        Self(WM::new().expect("No Wayland WM detected"))
    }
}

unsafe impl Sync for WaylandMouse {}
unsafe impl Send for WaylandMouse {}

impl MouseExt for WaylandMouse {
    fn get_pos(&mut self) -> Result<(i32, i32), crate::error::MousePosition> {
        self.0.get_pos()
    }

    fn get_physical_pos(&self) -> Result<(i32, i32), crate::error::MousePosition> {
        Err(crate::error::MousePosition::Unimplemented)
    }
}
