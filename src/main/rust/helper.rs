
use std::ffi;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublicError
{
    #[error("Application error: {0}")]
    ApplicationError (String),

    #[error("IO error: {0}")]
    IOError (String),
}

impl From<io::Error> for PublicError
{
    fn from (err: io::Error)
    -> PublicError
    {
        PublicError::IOError (err.to_string ())
    }
}

impl From<ffi::NulError> for PublicError
{
    fn from (err: ffi::NulError)
    -> PublicError
    {
        PublicError::ApplicationError (err.to_string ())
    }
}

