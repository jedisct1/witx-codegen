use std::fmt;

use witx::WitxError;

#[derive(Debug)]
pub enum Error {
    Witx(WitxError),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<WitxError> for Error {
    fn from(e: WitxError) -> Self {
        Self::Witx(e)
    }
}
