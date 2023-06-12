// Copyright (c) ZeroC, Inc.

use super::*;

#[derive(Debug)]
pub struct Compress {
    pub compress_args: bool,
    pub compress_return: bool,
}

impl Compress {
    pub fn parse_from(Unparsed { directive, args }: &Unparsed, span: &Span, reporter: &mut DiagnosticReporter) -> Self {
        debug_assert_eq!(directive, Self::directive());

        let (mut compress_args, mut compress_return) = (false, false);
        for arg in args {
            match arg.as_str() {
                "Args" => {
                    // TODO should we report a warning/error for duplicates?
                    compress_args = true;
                }
                "Return" => {
                    // TODO should we report a warning/error for duplicates?
                    compress_return = true;
                }
                _ => {
                    Diagnostic::new(Error::ArgumentNotSupported {
                        argument: arg.clone(),
                        directive: Self::directive().to_owned(),
                    })
                    .set_span(span)
                    .add_note("'Args' and 'Return' are the only valid arguments", None)
                    .report(reporter);
                }
            }
        }

        Compress { compress_args, compress_return }
    }

    pub fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter) {
        if !matches!(applied_on, Attributables::Operation(_) | Attributables::Interface(_)) {
            let note = "the compress attribute can only be applied to interfaces and operations";
            report_unexpected_attribute(self, span, Some(note), reporter);
        }
    }
}

implement_attribute_kind_for!(Compress, "compress", false);
