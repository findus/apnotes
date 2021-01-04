
#[derive(Debug)]
pub enum UpdateError {
    SyncError(String),
    IoError(String)
}


#[derive(Debug,PartialEq)]
pub enum NoteError {
    InsertionError(String),
    EditError(String),
    NeedsMerge,
    ContentNotChanged
}


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