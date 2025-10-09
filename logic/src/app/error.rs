use std::fmt::Display;

#[derive(Clone, Debug)]
pub enum ShaooohError {
    UnexpectedEndOfLoop,
    CommunicationError,
    ProcessingError,
}

impl Display for ShaooohError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::CommunicationError => "Communication Error",
            Self::UnexpectedEndOfLoop => "Unexpected End of Loop",
            Self::ProcessingError => "Processing Error",
        };
        write!(f, "{}", str)
    }
}
