use crate::code_block::CodeBlock;
use std::collections::HashMap;

use slice::grammar::{Module, NamedSymbol, ScopedSymbol};

#[derive(Debug)]
pub struct CodeMap {
    pub module_map: HashMap<String, Vec<CodeBlock>>,
}

impl CodeMap {
    pub fn new() -> CodeMap {
        CodeMap { module_map: HashMap::new() }
    }

    pub fn insert(&mut self, symbol: &dyn ScopedSymbol, code: CodeBlock) {
        let scope = symbol.scope().to_owned();
        match self.module_map.get_mut(&scope) {
            Some(vec) => vec.push(code),
            None => {
                self.module_map.insert(scope, vec![code]);
            }
        };
    }

    pub fn get(&self, module: &Module) -> Option<&Vec<CodeBlock>> {
        let scope = format!(
            "{}::{}",
            if module.is_top_level() { "" } else { module.scope() },
            module.identifier()
        );
        self.module_map.get(&scope)
    }
}
