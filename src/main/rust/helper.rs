
use std::ffi;
use std::io;
use std::str;
use std::string;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublicError
{
    #[error("Application error: {0}")]
    ApplicationError (String),

    #[error("IO error: {0}")]
    IOError (String),
}

impl From<ffi::FromBytesWithNulError> for PublicError
{
    fn from (err: ffi::FromBytesWithNulError)
    -> PublicError
    {
        PublicError::ApplicationError (err.to_string ())
    }
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

impl From<str::Utf8Error> for PublicError
{
    fn from (err: str::Utf8Error)
    -> PublicError
    {
        PublicError::ApplicationError (err.to_string ())
    }
}


impl From<string::FromUtf8Error> for PublicError
{
    fn from (err: string::FromUtf8Error)
    -> PublicError
    {
        PublicError::ApplicationError (err.to_string ())
    }
}

