use failure::Error;
use failure_derive::Fail;

#[derive(Debug, Fail)]
pub enum NetworkError {
    #[fail(display = "Fatal error: {}", _0)]
    FatalError(Error),
    #[fail(display = "Connection reset")]
    RebuildRequired,
}