use thiserror::Error;

#[derive(Debug, Error)]
pub enum MousePosition {
    #[error("This function or feature is not implemented")]
    Unimplemented,
    #[error("Mouse position could not be correctly extracted")]
    BadExtract,
    #[error("No cursor found")]
    NoMouseFound,
}
