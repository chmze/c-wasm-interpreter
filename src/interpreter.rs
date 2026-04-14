use crate::parser::Parser;

pub struct Interpreter {

}

impl Interpreter {
    pub fn new() -> Self {
        let mut parser = Parser::new("s");
        let res = parser.parse();
        Self {}
    }
}
