#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ParseErrorKind {
    NoHeaderFound,
    EndOfBufferNoTrailer,
    ProtocolError,
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseErrorKind {}

#[derive(Debug)]
pub(crate) struct ParseError {
    kind: ParseErrorKind,
    message: Box<str>,
    index: usize,
}

impl ParseError {
    pub(crate) fn new(kind: ParseErrorKind, message: &str, index: usize) -> Self {
        Self {
            kind,
            message: message.into(),
            index,
        }
    }

    pub(crate) fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn err_index(&self) -> usize {
        self.index
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{kind}: {message}",
            kind = self.kind,
            message = self.message
        )
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}
