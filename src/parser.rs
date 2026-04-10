use crate::lexer::*;

pub struct ASTRoot {
    pub statements: Vec<ASTNode>,
}

pub enum ASTDataType {
    Int,
}

pub struct ASTFunc {
    pub ty: ASTDataType,
}

pub enum ASTNodeType {
    Root(ASTRoot),
    Func(ASTFunc),
    EOF,
}

pub struct ASTNode {
    pub ty: ASTNodeType,
}

impl ASTNode {
    pub fn new(ty: ASTNodeType) -> Self {
        ASTNode { ty }
    }
}

pub struct Parser {
    tokens: Vec<LexToken>,
    pos: usize,
    checkpoint: usize,
}

impl Parser {
    pub fn new(s: &str) -> Self {
        let mut lexer = Lexer::new(s);
        Self { tokens: lexer.read(), pos: 0, checkpoint: 0 }
    }

    pub fn new_lexed(tokens: Vec<LexToken>) -> Self {
        Self { tokens, pos: 0, checkpoint: 0 }
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn set_checkpoint(&mut self, pos: usize) {
        self.checkpoint = pos;
    }

    fn return_checkpoint(&mut self) {
        self.set_pos(self.checkpoint);
    }

    fn current(&self) -> &LexToken {
        if self.pos >= self.tokens.len() {
            return self.tokens.last().unwrap();
        }

        &self.tokens[self.pos]
    }

    fn read(&mut self) -> &LexToken {
        self.pos += 1;
        self.current()
    }

    fn try_parse_type(&mut self) -> Option<ASTDataType> {
        if self.current().ty == LexTokenType::Int {
            return Some(ASTDataType::Int);
        }

        None
    }

    fn try_parse_func(&mut self) -> Option<ASTNode> {
        let ty = self.try_parse_type()?;

        Some(ASTNode::new(ASTNodeType::Func(ASTFunc {
            ty
        })))
    }

    fn parse_current(&mut self) -> ASTNode {
        if self.current().ty == LexTokenType::EOF {
            return ASTNode::new(ASTNodeType::EOF);
        }

        self.set_checkpoint(self.pos);
        if let Some(node) = self.try_parse_func() {
            self.return_checkpoint();
            return node;
        }

        ASTNode::new(ASTNodeType::EOF)
    }

    pub fn parse(&mut self) -> ASTNode {
        let mut root = ASTRoot { statements: Vec::new() };

        let cur = self.parse_current();
        root.statements.push(cur);

        let root = ASTNode::new(ASTNodeType::Root(root));
        root
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::*;

    #[test]
    fn test1() {
        let s = "int main() { int a = 1; }";

        let mut parser = Parser::new(s);
        parser.parse();
    }
}
