
#[derive(Debug,PartialEq)]
pub enum UpdateError {
    SyncError(String),
    IoError(String)
}


#[derive(Debug,PartialEq)]
pub enum NoteError {
    InsertionError(String),
    EditError(String),
    NeedsMerge,
    ContentNotChanged,
    NoteNotFound
}

impl std::error::Error for NoteError {}
impl std::error::Error for UpdateError {}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for NoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}