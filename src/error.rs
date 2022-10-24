#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    LeadingContent,
    TrailingContent,
    EndTagWithData,
    StartTagWithoutData,
    ReentrantFragment,
    UnbalancedTags,
}

impl Error {
    pub fn at(self, line: usize) -> ErrorWithLine {
        ErrorWithLine(line, self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeadingContent => write!(f, "Error::LeadingContent"),
            Self::TrailingContent => write!(f, "Error::TrailingContent"),
            Self::EndTagWithData => write!(f, "Error::EndTagWithData"),
            Self::StartTagWithoutData => write!(f, "Error::StartTagWithoutData"),
            Self::ReentrantFragment => write!(f, "Error::ReentrantFragment"),
            Self::UnbalancedTags => write!(f, "Error::UnbalancedTags"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorWithLine(pub usize, pub Error);

impl std::fmt::Display for ErrorWithLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}", self.1, self.0 + 1)
    }
}

impl std::error::Error for ErrorWithLine {}
