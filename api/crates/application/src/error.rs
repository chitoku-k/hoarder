pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
    #[source]
    error: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error {
            kind,
            error: Some(error.into()),
        }
    }

    pub fn other<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::new(ErrorKind::Other, error)
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn error(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.error.as_deref()
    }

    pub fn into_inner(self) -> (ErrorKind, Option<Box<dyn std::error::Error + Send + Sync + 'static>>) {
        (self.kind, self.error)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            kind,
            error: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error("the server was unable to start")]
    ServerStartFailed,

    #[error("the server was unable to bind")]
    ServerBindFailed,

    #[cfg(feature = "tls")]
    #[error("the server certificate was invalid")]
    ServerCertificateInvalid { cert: String, key: String },

    #[error("other error")]
    Other,
}
