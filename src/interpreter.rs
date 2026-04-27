use std::collections::HashMap;

use crate::parser::{ASTData, ASTDataType, ASTFunc, ASTNode, ASTNodeType, ASTRoot, ASTVar, Parser};

pub struct EnvVar {
    address: usize,
}

impl EnvVar {
    pub fn new(address: usize) -> Self {
        Self { address }
    }
}

pub struct EnvFunc {

}

impl EnvFunc {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct Memory {
    data: [u8; 65536],
    allocations: HashMap<u16, (u16, ASTDataType)>,
}

impl Memory {
    pub fn new() -> Self {
        Self { data: [0; 65536], allocations: HashMap::new() }
    }

    pub fn write(&mut self, addr: u16, ty: ASTData) {

    }

    pub fn read(&self, addr: u16) {

    }
}

pub struct Environment {
    memory: Memory,
    vars: HashMap<String, EnvVar>,
    funcs: HashMap<String, EnvFunc>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { memory: Memory::new(), vars: HashMap::new(), funcs: HashMap::new() }
    }
}

pub struct Exec {} // execution result, fill in later

pub struct Interpreter {
    root: ASTNode,
    env: Environment,
}

impl Interpreter {
    pub fn new(s: &str) -> Self {
        let mut parser = Parser::new(s);
        let root = parser.parse();

        Self { root, env: Environment::new() }
    }

    pub fn new_with_node(root: ASTNode) -> Self {
        Self { root, env: Environment::new() }
    }

    fn exec_root(&mut self, root: ASTRoot) {

    }

    fn exec_func(&mut self, func: ASTFunc) {
        // TODO for now
    }

    fn exec_var(&mut self, var: ASTVar) {
    }

    fn node_eval(&mut self, node: ASTNode) {
        match node.ty {
            ASTNodeType::Root(root) => self.exec_root(root),
            ASTNodeType::Func(func) => self.exec_func(func),
            ASTNodeType::Var(var) => self.exec_var(var),
            ASTNodeType::EOF => (),
        }
    }

    pub fn execute(&mut self) -> Option<Exec> {

        self.node_eval(self.root.clone());
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::*;

    #[test]
    pub fn test() {
        let mut i = Interpreter::new("int a = 1; int b = a + 1; int c = a + b * 2;");
        assert!(i.execute().is_some());
    }

}
