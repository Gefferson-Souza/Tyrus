use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum OxidizerError {
    #[error("IO Error: {0}")]
    #[diagnostic(code(oxidizer::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Parsing Error: {message}")]
    #[diagnostic(code(oxidizer::parse_error))]
    ParserError {
        message: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("{message}")]
        span: SourceSpan,
    },

    #[error("Lint Error: Rust does not support 'var'. Use 'let' or 'const'.")]
    #[diagnostic(code(oxidizer::lint::no_var))]
    UseOfVar {
        #[source_code]
        src: NamedSource<String>,
        #[label("replace 'var' with 'let' or 'const'")]
        span: SourceSpan,
    },

    #[error("Lint Error: Rust requires strict typing. 'any' is not allowed.")]
    #[diagnostic(code(oxidizer::lint::no_any))]
    UseOfAny {
        #[source_code]
        src: NamedSource<String>,
        #[label("specify a concrete type")]
        span: SourceSpan,
    },

    #[error("Lint Error: Code injection via 'eval' is unsafe and not supported in Rust.")]
    #[diagnostic(code(oxidizer::lint::no_eval))]
    UseOfEval {
        #[source_code]
        src: NamedSource<String>,
        #[label("remove 'eval' usage")]
        span: SourceSpan,
    },

    #[error("Unsupported Feature: {feature} is not yet supported in Oxidizer.")]
    #[diagnostic(code(oxidizer::unsupported))]
    UnsupportedFeature {
        feature: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("this feature is pending implementation")]
        span: SourceSpan,
    },

    #[error("Formatting Error: {0}")]
    #[diagnostic(code(oxidizer::fmt_error))]
    FormattingError(String),

    #[error("Unknown Error")]
    #[diagnostic(code(oxidizer::unknown))]
    Unknown,
}
