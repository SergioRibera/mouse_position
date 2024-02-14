use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum MousePosition {
    IO(#[from] std::io::Error),
    Utf8(#[from] std::string::FromUtf8Error),
    ParseInt(#[from] std::num::ParseIntError),
    #[error("This function or feature is not implemented")]
    Unimplemented,
    #[error("Mouse position could not be correctly extracted")]
    BadExtract,
    #[error("No cursor found")]
    NoMouseFound,
    #[error("Socket Not Found")]
    SocketNotFound,
    #[error("WM not detected")]
    WMNotDetected,
}
