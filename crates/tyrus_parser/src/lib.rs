use miette::{NamedSource, SourceSpan};
use std::path::Path;
use tyrus_diagnostics::OxidizerError;

use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap, Spanned,
};
use swc_ecma_ast::Program;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};

pub fn parse(path: &Path) -> Result<Program, OxidizerError> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm.load_file(path).map_err(OxidizerError::IoError)?;

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: path.to_string_lossy().ends_with(".tsx"),
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    match parser.parse_program() {
        Ok(program) => Ok(program),
        Err(e) => {
            // Convert SWC error to OxidizerError
            let span = e.span();
            let message = e.into_kind().msg().to_string();

            // Calculate start and length for SourceSpan
            let start = span.lo.0 as usize - 1;
            let end = span.hi.0 as usize - 1;
            let len = end - start;

            Err(OxidizerError::ParserError {
                message,
                src: NamedSource::new(path.to_string_lossy(), fm.src.to_string()),
                span: SourceSpan::new(start.into(), len),
            })
        }
    }
}
