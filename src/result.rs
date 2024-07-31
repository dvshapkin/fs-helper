use std::io;
use std::sync::mpsc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    File,
    Channel
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    cause: Box<dyn std::error::Error>
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error {
            kind: ErrorKind::File,
            cause: Box::new(e)
        }
    }
}

impl<T: 'static> From<mpsc::SendError<T>> for Error {
    fn from(e: mpsc::SendError<T>) -> Error {
        Error {
            kind: ErrorKind::Channel,
            cause: Box::new(e)
        }
    }
}

impl<T> From<Error> for Result<T> {
    fn from(e: Error) -> Result<T> {
        Err(e)
    }
}
