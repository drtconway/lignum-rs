
pub type Result<T> = std::result::Result<T, LignumError>;

#[derive(Debug)]
pub enum LignumError {
    Backend(Box<dyn std::error::Error + Send + Sync>),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for LignumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LignumError::Backend(_) => write!(f, "Lignum encountered a backend error"),
            LignumError::Other(_) => write!(f, "Lignum encountered an error"),
        }
    }
}

impl std::error::Error for LignumError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LignumError::Backend(err) => Some(err.as_ref()),
            LignumError::Other(err) => Some(err.as_ref()),
        }
    }
}

impl From<std::io::Error> for LignumError {
    fn from(err: std::io::Error) -> Self {
        LignumError::Other(Box::new(err))
    }
}

#[cfg(feature = "cairo")]
impl From<cairo::Error> for LignumError {
    fn from(err: cairo::Error) -> Self {
        LignumError::Backend(Box::new(err))
    }
}