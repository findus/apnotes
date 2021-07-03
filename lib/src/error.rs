use xdg::BaseDirectoriesError;

pub type Result<T> = std::result::Result<T, Box<dyn ErrorCode>>;


pub trait ErrorCode: std::error::Error {
    fn error_code(&self) -> i32;
    fn human_readable_error_message(&self) -> String;
}

#[derive(Debug,PartialEq)]
pub enum ProfileError {
    NotFound(String),
    NoPasswordProvided(),
    AgentLocked(),
}

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


impl ErrorCode for dyn std::error::Error {
    fn error_code(&self) -> i32 {
        return 255
    }

    fn human_readable_error_message(&self) -> String {
        "A general error occurred:".to_string()
    }
}

impl ErrorCode for ProfileError {
    fn error_code(&self) -> i32 {
        match self {
            ProfileError::NotFound(_) => { 1 }
            ProfileError::NoPasswordProvided() => { 2 }
            ProfileError::AgentLocked() => { 3 }
        }
    }

    fn human_readable_error_message(&self) -> String {
        "An error occurred while loading your config file:".to_string()
    }
}

impl ErrorCode for UpdateError {
    fn error_code(&self) -> i32 {
        match self {
            UpdateError::SyncError(_) => { 20 }
            UpdateError::IoError(_) => { 21 }
        }
    }

    fn human_readable_error_message(&self) -> String {
        "An error occurred while updating your notes:".to_string()
    }
}

impl ErrorCode for NoteError {
    fn error_code(&self) -> i32 {
        match self {
            NoteError::InsertionError(_) => { 30 }
            NoteError::EditError(_) => { 31 }
            NoteError::NeedsMerge => { 32 }
            NoteError::ContentNotChanged => { 33 }
            NoteError::NoteNotFound => { 34 }
        }
    }

    fn human_readable_error_message(&self) -> String {
        "An error occurred while editing your notes:".to_string()
    }
}

impl ErrorCode for diesel::result::Error {
    fn error_code(&self) -> i32 {
        return 255;
    }

    fn human_readable_error_message(&self) -> String {
        "A database error occurred:".to_string()
    }
}

impl ErrorCode for std::str::Utf8Error {
    fn error_code(&self) -> i32 {
        return 255;
    }

    fn human_readable_error_message(&self) -> String {
        "An encoding error occurred:".to_string()
    }
}

impl ErrorCode for xdg::BaseDirectoriesError {
    fn error_code(&self) -> i32 {
        return 255;
    }

    fn human_readable_error_message(&self) -> String {
        "An error occurred while interacting with your user home:".to_string()
    }
}

impl ErrorCode for std::io::Error {
    fn error_code(&self) -> i32 {
        return 255;
    }

    fn human_readable_error_message(&self) -> String {
        "A general io error occurred".to_string()
    }
}

impl ErrorCode for regex::Error {
    fn error_code(&self) -> i32 {
        return 255;
    }

    fn human_readable_error_message(&self) -> String {
        "A parsing error occurred:".to_string()
    }
}

impl ErrorCode for imap::Error{
    fn error_code(&self) -> i32 {
        return 255;
    }

    fn human_readable_error_message(&self) -> String {
        "An error occurred while messaging your mail server:".to_string()
    }
}

impl ErrorCode for secret_service::SsError {
    fn error_code(&self) -> i32 {
        return 3;
    }

    fn human_readable_error_message(&self) -> String {
        "An error occurred while contacting the password store:".to_string()
    }
}

impl std::convert::From<NoteError> for std::boxed::Box<dyn ErrorCode> {
    fn from(e: NoteError) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<UpdateError> for std::boxed::Box<dyn ErrorCode> {
    fn from(e: UpdateError) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<secret_service::SsError> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: secret_service::SsError) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<diesel::result::Error> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: diesel::result::Error) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<ProfileError> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: ProfileError) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<std::str::Utf8Error> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: std::str::Utf8Error) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<BaseDirectoriesError> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: BaseDirectoriesError) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<std::io::Error> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: std::io::Error) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<regex::Error> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: regex::Error) -> Self {
        Box::new(e)
    }
}

impl std::convert::From<imap::Error> for  std::boxed::Box<dyn ErrorCode> {
    fn from(e: imap::Error) -> Self {
        Box::new(e)
    }
}

impl std::error::Error for ProfileError {}
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

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}