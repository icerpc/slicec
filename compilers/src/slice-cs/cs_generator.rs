
use slice::grammar::*;
use slice::visitor::*;

struct CodeWriter {}

struct CsVisitor {
    output: CodeWriter,
}
