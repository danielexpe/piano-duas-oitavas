#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("falha de terminal: {0}")]
    Terminal(#[from] std::io::Error),

    #[error("backend indisponível: {0}")]
    BackendUnavailable(&'static str),

    #[error("recurso indisponível nesta fase: {0}")]
    NotImplemented(&'static str),

    #[error("opções inválidas: {0}")]
    InvalidOptions(&'static str),

    #[error("falha ao instalar tratadores de sinal: {0}")]
    Signals(String),
}

impl AppError {
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::BackendUnavailable(_) => 3,
            Self::Terminal(_)
            | Self::NotImplemented(_)
            | Self::InvalidOptions(_)
            | Self::Signals(_) => 1,
        }
    }
}
