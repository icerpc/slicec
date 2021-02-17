
//------------------------------------------------------------------------------
// Node
//------------------------------------------------------------------------------
/// Base trait that all grammar elements implement.
trait Node {
    fn visit(&self, visitor: &mut impl Visitor, ast: &SliceAst);
    fn get_kind(&self) -> NodeKind;
    fn get_location(&self) -> Location;
}

//------------------------------------------------------------------------------
// NodeKind
//------------------------------------------------------------------------------
/// Enum defining all the different kinds of grammar elements.
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
enum NodeKind {
    Module,
    Struct,
    Interface,
    DataMember,
    Identifier,
    TypeUse,
}

//------------------------------------------------------------------------------
// Definition
//------------------------------------------------------------------------------
/// Base trait that all slice definitions implement.
/// All Definitions are stored by index in the AST vector.
trait Definition: Node {
    fn get_id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}

