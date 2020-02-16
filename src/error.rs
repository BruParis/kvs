use failure::Fail;
use std::fs::File;
use std::io;
use std::str;

#[derive(Fail, Debug)]
pub enum KVError {
    #[fail(display = "An I/O error occurred: {}", error)]
    IO { error: io::Error },
    #[fail(display = "A serde error occurred: {}", error)]
    Serde { error: serde_json::error::Error },
    #[fail(display = "An into inner error with the writer occured: {}", error)]
    Writer {
        error: io::IntoInnerError<io::BufWriter<File>>,
    },
    #[fail(display = "An Utf8 error happened with the reader: {}", error)]
    Utf8 { error: str::Utf8Error },
    #[fail(display = "An String Utf8 error happened: {}", error)]
    StringUtf8 { error: std::string::FromUtf8Error },
    #[fail(display = "An error occured with sled engine: {}", error)]
    Sled { error: sled::Error },
    #[fail(display = "Wrong engine.")]
    WrongEngine,
    #[fail(display = "Fail to get value from {}", _0)]
    FailGet(String),
    #[fail(display = "Error reading entry from log file")]
    ReadLog,
    #[fail(display = "An error occurred.")]
    FailSet,
    #[fail(display = "An error occurred.")]
    FailRemove,
    #[fail(display = "{}", _0)]
    EngineNotFound(String),
    #[fail(display = "An error occurred.")]
    None,
}

impl From<io::Error> for KVError {
    fn from(err: io::Error) -> KVError {
        KVError::IO { error: err }
    }
}

impl From<str::Utf8Error> for KVError {
    fn from(err: str::Utf8Error) -> KVError {
        KVError::Utf8 { error: err }
    }
}

impl From<std::string::FromUtf8Error> for KVError {
    fn from(err: std::string::FromUtf8Error) -> KVError {
        KVError::StringUtf8 { error: err }
    }
}

impl From<io::IntoInnerError<io::BufWriter<File>>> for KVError {
    fn from(err: io::IntoInnerError<io::BufWriter<File>>) -> KVError {
        KVError::Writer { error: err }
    }
}

impl From<serde_json::error::Error> for KVError {
    fn from(err: serde_json::error::Error) -> KVError {
        KVError::Serde { error: err }
    }
}

impl From<sled::Error> for KVError {
    fn from(err: sled::Error) -> KVError {
        KVError::Sled { error: err }
    }
}

pub type Result<T> = std::result::Result<T, KVError>;
