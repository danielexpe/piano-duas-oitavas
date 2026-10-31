#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("falha de terminal: {0}")]
    Terminal(#[from] std::io::Error),

    #[error("backend indisponível: {0}")]
    BackendUnavailable(&'static str),

    #[error("backend indisponível: {0}")]
    FbUnavailable(String),

    #[error("falha no framebuffer: {0}")]
    Fb(String),

    #[error("falha na captura: {0}")]
    Capture(String),

    #[error("recurso embutido inválido: {0}")]
    Asset(&'static str),

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
            Self::BackendUnavailable(_) | Self::FbUnavailable(_) => 3,
            Self::Terminal(_)
            | Self::Fb(_)
            | Self::Capture(_)
            | Self::Asset(_)
            | Self::NotImplemented(_)
            | Self::InvalidOptions(_)
            | Self::Signals(_) => 1,
        }
    }
}
