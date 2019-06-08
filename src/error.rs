use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    InvalidArg(&'static str),
    LogFileLimitExceeded,
    IOError(IOError),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Error {
        Error::IOError(err)
    }
}
