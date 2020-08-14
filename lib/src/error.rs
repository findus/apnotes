
#[derive(Debug)]
pub enum UpdateError {
    SyncError(String),
    EditError(String),
    IoError(String)
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}