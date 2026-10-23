//! Central error type ([`TyrusError`]) and stable error taxonomy
//! ([`ErrorCategory`], `TYRUS-EXXXX` codes — Rule 14) for the whole pipeline,
//! rendered through `miette` for terminal diagnostics.
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::missing_errors_doc)]
#![allow(unused_assignments)]
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

/// Pipeline stage a [`TyrusError`] originates from (Rule 14, ADR 0013).
/// Serialized into the `--json` envelope as a lowercase string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorCategory {
    /// Lexing/parsing of the TypeScript source (SWC).
    Parse,
    /// Semantic analysis: lint rules and unsupported-API detection.
    Analyze,
    /// Rust code generation (reserved — no codegen-stage errors yet).
    Codegen,
    /// Filesystem/input errors, including CLI input validation.
    Io,
    /// Formatting of the generated Rust source.
    Format,
    /// Front-end sidecar (project TypeScript resolution, emit, facts).
    Frontend,
}

impl ErrorCategory {
    /// Lowercase name used in the `--json` envelope.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Analyze => "analyze",
            Self::Codegen => "codegen",
            Self::Io => "io",
            Self::Format => "format",
            Self::Frontend => "frontend",
        }
    }
}

impl TyrusError {
    /// Stable machine-readable code (Rule 14). Codes never change meaning or
    /// get reused; the leading digit encodes the category block:
    /// `E0xxx` parse, `E1xxx` analyze, `E2xxx` codegen (reserved),
    /// `E3xxx` io/input, `E4xxx` format, `E7xxx` front-end, `E9999` unknown.
    #[must_use]
    pub fn stable_code(&self) -> &'static str {
        match self {
            Self::ParserError { .. } => "TYRUS-E0001",
            Self::UseOfVar { .. } => "TYRUS-E1001",
            Self::UseOfAny { .. } => "TYRUS-E1002",
            Self::UseOfEval { .. } => "TYRUS-E1003",
            Self::UnsupportedFeature { .. } => "TYRUS-E1004",
            Self::AmbiguousMainEntrypoint { .. } => "TYRUS-E1005",
            Self::IoError(_) => "TYRUS-E3001",
            Self::Validation(_) => "TYRUS-E3002",
            Self::FormattingError(_) => "TYRUS-E4001",
            Self::MissingProjectTypescript { .. } => "TYRUS-E7001",
            Self::Unknown => "TYRUS-E9999",
        }
    }

    /// Category block of [`Self::stable_code`]. `Unknown` maps to `Io` as the
    /// catch-all for failures with no attributable pipeline stage.
    #[must_use]
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ParserError { .. } => ErrorCategory::Parse,
            Self::UseOfVar { .. }
            | Self::UseOfAny { .. }
            | Self::UseOfEval { .. }
            | Self::UnsupportedFeature { .. }
            | Self::AmbiguousMainEntrypoint { .. } => ErrorCategory::Analyze,
            Self::IoError(_) | Self::Validation(_) | Self::Unknown => ErrorCategory::Io,
            Self::FormattingError(_) => ErrorCategory::Format,
            Self::MissingProjectTypescript { .. } => ErrorCategory::Frontend,
        }
    }
}

/// Every failure the Tyrus pipeline can produce. Each variant carries a
/// stable code ([`Self::stable_code`]) and category ([`Self::category`]).
#[derive(Error, Diagnostic, Debug)]
pub enum TyrusError {
    #[error("IO Error: {0}")]
    #[diagnostic(code(tyrus::io_error))]
    /// Underlying filesystem/IO failure (`TYRUS-E3001`).
    IoError(#[from] std::io::Error),

    #[error("Parsing Error: {message}")]
    #[diagnostic(code(tyrus::parse_error))]
    /// SWC could not parse the input (`TYRUS-E0001`).
    ParserError {
        /// Parser-provided description of the syntax problem.
        message: String,
        /// Source file the diagnostic points into.
        #[source_code]
        src: NamedSource<String>,
        /// Offending region within `src`.
        #[label("{message}")]
        span: SourceSpan,
    },

    #[error("Lint Error: Rust does not support 'var'. Use 'let' or 'const'.")]
    #[diagnostic(code(tyrus::lint::no_var))]
    /// Lint: `var` declaration (`TYRUS-E1001`).
    UseOfVar {
        /// Source file the diagnostic points into.
        #[source_code]
        src: NamedSource<String>,
        /// Offending region within `src`.
        #[label("replace 'var' with 'let' or 'const'")]
        span: SourceSpan,
    },

    #[error("Lint Error: Rust requires strict typing. 'any' is not allowed.")]
    #[diagnostic(code(tyrus::lint::no_any))]
    /// Lint: `any` type (`TYRUS-E1002`).
    UseOfAny {
        /// Source file the diagnostic points into.
        #[source_code]
        src: NamedSource<String>,
        /// Offending region within `src`.
        #[label("specify a concrete type")]
        span: SourceSpan,
    },

    #[error("Lint Error: Code injection via 'eval' is unsafe and not supported in Rust.")]
    #[diagnostic(code(tyrus::lint::no_eval))]
    /// Lint: `eval` call (`TYRUS-E1003`).
    UseOfEval {
        /// Source file the diagnostic points into.
        #[source_code]
        src: NamedSource<String>,
        /// Offending region within `src`.
        #[label("remove 'eval' usage")]
        span: SourceSpan,
    },

    #[error("Unsupported Feature: {feature} is not yet supported in Tyrus.")]
    #[diagnostic(code(tyrus::unsupported))]
    /// Construct outside the Oxidizable subset (`TYRUS-E1004`).
    UnsupportedFeature {
        /// Human-readable name of the unsupported construct.
        feature: String,
        /// Source file the diagnostic points into.
        #[source_code]
        src: NamedSource<String>,
        /// Offending region within `src`.
        #[label("this feature is pending implementation")]
        span: SourceSpan,
    },

    #[error(
        "Lint Error: Ambiguous main entrypoint — file declares `function main()` AND \
         contains top-level executable statements. Tyrus would either drop the \
         statements or duplicate the main function. Choose one: either move the \
         statements inside `main()`, or remove the user-declared `main` function."
    )]
    #[diagnostic(code(tyrus::lint::ambiguous_main_entrypoint))]
    /// `function main()` plus top-level statements in one file (`TYRUS-E1005`).
    AmbiguousMainEntrypoint {
        /// Source file the diagnostic points into.
        #[source_code]
        src: NamedSource<String>,
        /// Offending region within `src`.
        #[label("first top-level statement here")]
        span: SourceSpan,
    },

    #[error("Formatting Error: {0}")]
    #[diagnostic(code(tyrus::fmt_error))]
    /// `prettyplease`/`syn` rejected the generated source (`TYRUS-E4001`).
    FormattingError(String),

    #[error("Validation Error: {0}")]
    #[diagnostic(code(tyrus::validation_error))]
    /// CLI/input validation failure (`TYRUS-E3002`).
    Validation(String),

    #[error(
        "Front-end Error: o projeto em {root} não tem `typescript` local; \
         instale a versão do projeto (o Tyrus usa o TypeScript do próprio projeto)."
    )]
    #[diagnostic(code(tyrus::frontend::missing_typescript))]
    /// Project has no local `typescript` package (`TYRUS-E7001`).
    MissingProjectTypescript {
        /// Project root where `typescript` was looked up.
        root: String,
    },

    #[error("Unknown Error")]
    #[diagnostic(code(tyrus::unknown))]
    /// Catch-all with no attributable stage (`TYRUS-E9999`).
    Unknown,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn named(src: &str) -> NamedSource<String> {
        NamedSource::new("test.ts", src.to_string())
    }

    fn span() -> SourceSpan {
        (0, 1).into()
    }

    /// One instance of every variant. Adding a variant without extending this
    /// list still fails the build: `stable_code`/`category` use exhaustive
    /// matches, so the compiler enforces coverage; this list only feeds the
    /// uniqueness/format assertions below.
    fn all_variants() -> Vec<TyrusError> {
        vec![
            TyrusError::IoError(std::io::Error::other("io")),
            TyrusError::ParserError {
                message: "m".into(),
                src: named("x"),
                span: span(),
            },
            TyrusError::UseOfVar {
                src: named("var x;"),
                span: span(),
            },
            TyrusError::UseOfAny {
                src: named("let a: any;"),
                span: span(),
            },
            TyrusError::UseOfEval {
                src: named("eval('')"),
                span: span(),
            },
            TyrusError::UnsupportedFeature {
                feature: "f".into(),
                src: named("x"),
                span: span(),
            },
            TyrusError::AmbiguousMainEntrypoint {
                src: named("x"),
                span: span(),
            },
            TyrusError::FormattingError("f".into()),
            TyrusError::Validation("v".into()),
            TyrusError::MissingProjectTypescript {
                root: "/project".into(),
            },
            TyrusError::Unknown,
        ]
    }

    #[test]
    fn stable_codes_are_unique() {
        let variants = all_variants();
        let codes: HashSet<&str> = variants.iter().map(TyrusError::stable_code).collect();
        assert_eq!(codes.len(), variants.len(), "duplicate stable code");
    }

    #[test]
    fn missing_project_typescript_has_the_front_end_code() {
        let error = TyrusError::MissingProjectTypescript {
            root: "/project".into(),
        };
        assert_eq!(error.stable_code(), "TYRUS-E7001");
        assert_eq!(error.category(), ErrorCategory::Frontend);
        assert_eq!(error.category().as_str(), "frontend");
    }

    #[test]
    fn stable_codes_follow_the_format() {
        for e in all_variants() {
            let code = e.stable_code();
            let digits = code.strip_prefix("TYRUS-E").expect("TYRUS-E prefix");
            assert_eq!(digits.len(), 4, "{code}: expected 4 digits");
            assert!(digits.chars().all(|c| c.is_ascii_digit()), "{code}");
        }
    }

    #[test]
    fn code_block_matches_category() {
        for e in all_variants() {
            let block = e
                .stable_code()
                .strip_prefix("TYRUS-E")
                .and_then(|d| d.chars().next())
                .expect("leading digit");
            let expected = match e.category() {
                ErrorCategory::Parse => '0',
                ErrorCategory::Analyze => '1',
                ErrorCategory::Codegen => '2',
                ErrorCategory::Io => '3',
                ErrorCategory::Format => '4',
                ErrorCategory::Frontend => '7',
            };
            // Unknown (E9999) is the deliberate exception in the Io catch-all.
            if e.stable_code() != "TYRUS-E9999" {
                assert_eq!(block, expected, "{}", e.stable_code());
            }
        }
    }
}
