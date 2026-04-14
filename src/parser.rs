use crate::lexer::*;

#[derive(Debug, Clone)]
pub struct ASTRoot {
    pub statements: Vec<ASTNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ASTDataType {
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
}

#[derive(Debug, Clone)]
pub struct ASTData {
    pub signed: bool,
    pub ty: ASTDataType,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ASTIdentifier {
    pub literal: String,
}

#[derive(Debug, Clone)]
pub struct ASTVar {
    pub ty: ASTData,
    pub name: ASTIdentifier,
}

#[derive(Debug, Clone)]
pub struct ASTFuncParam {
    pub ty: ASTData,
    pub name: ASTIdentifier,
}

#[derive(Debug, Clone)]
pub struct ASTFunc {
    pub ty: ASTData,
    pub name: ASTIdentifier,
    pub params: Vec<ASTFuncParam>,
    pub body: Vec<ASTNode>,
}

#[derive(Debug, Clone)]
pub enum ASTNodeType {
    Root(ASTRoot),
    Var(ASTVar),
    Func(ASTFunc),
    EOF,
}

#[derive(Debug, Clone)]
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

    fn token_at(&self, pos: usize) -> &LexToken {
        if pos >= self.tokens.len() {
            return self.tokens.last().unwrap();
        }

        &self.tokens[pos]
    }

    fn current(&self) -> &LexToken {
        self.token_at(self.pos)
    }

    fn read(&mut self) -> &LexToken {
        self.pos += 1;
        self.current()
    }

    fn peek(&self) -> &LexToken {
        self.token_at(self.pos + 1)
    }

    fn try_parse_current(&mut self, ty: LexTokenType) -> Option<&LexToken> {
        if self.current().ty == ty {
            self.read();
            Some(self.token_at(self.pos-1))
        } else {
            None
        }
    }

    fn try_parse_peek(&mut self, ty: LexTokenType) -> Option<&LexToken> {
        if self.peek().ty == ty {
            Some(self.read())
        } else {
            None
        }
    }

    fn try_parse_identifier(&mut self) -> Option<ASTIdentifier> {
        let current = self.current();

        match current.ty {
            LexTokenType::Identifier => {
                let literal = current.literal.clone();
                self.read();

                Some(ASTIdentifier { literal })
            }
            _ => None,
        }
    }

    fn get_default_signedness(&self, ty: ASTDataType) -> bool {
        true
    }

    fn try_parse_type(&mut self) -> Option<ASTData> {
        let explicit_signed = match self.try_parse_current(LexTokenType::Signed) {
            Some(_) => Some(true),
            None => {
                if self.try_parse_current(LexTokenType::Unsigned).is_some() {
                    Some(false)
                } else {
                    None
                }
            }
        };

        let ty = match self.current().ty {
            LexTokenType::Char => ASTDataType::Char,
            LexTokenType::Short => ASTDataType::Short,
            LexTokenType::Int => ASTDataType::Int,
            LexTokenType::Long => {
                match self.try_parse_peek(LexTokenType::Long) {
                    Some(_) =>  ASTDataType::LongLong,
                    None => ASTDataType::Long,
                }
            }
            LexTokenType::Float => ASTDataType::Float,
            LexTokenType::Double => ASTDataType::Double,
            _ if explicit_signed.is_some() => ASTDataType::Int,
            _ => return None,
        };

        if explicit_signed.is_none() || self.current().ty != LexTokenType::Identifier {
            self.read();
        }

        if ty == ASTDataType::Short || ty == ASTDataType::Long || ty == ASTDataType::LongLong {
            _ = self.try_parse_current(LexTokenType::Int); // TODO: add error handling
        }

        let signed = explicit_signed.unwrap_or(self.get_default_signedness(ty));

        Some(ASTData { signed, ty })
    }

    fn try_parse_func_params(&mut self) -> Option<Vec<ASTFuncParam>> {
        let _ = self.try_parse_current(LexTokenType::LParen)?;
        let mut params = Vec::new();

        if self.try_parse_current(LexTokenType::RParen).is_some() {
            return Some(params);
        }

        loop {
            let ty = self.try_parse_type()?;
            let name = self.try_parse_identifier()?;

            params.push(ASTFuncParam { ty, name });

            if self.try_parse_current(LexTokenType::Comma).is_none() {
                let _ = self.try_parse_current(LexTokenType::RParen)?;
                break;
            }
        }

        Some(params)
    }

    fn try_parse_brace(&mut self) -> Option<Vec<ASTNode>> {
        let _ = self.try_parse_current(LexTokenType::LBrace)?;
        // let body = ;
        let _ = self.try_parse_current(LexTokenType::RBrace)?;

        None
    }

    fn try_parse_func(&mut self) -> Option<ASTNode> {
        let ty = self.try_parse_type()?;
        let name = self.try_parse_identifier()?;
        let params = self.try_parse_func_params()?;
        let body = self.try_parse_brace()?;

        Some(ASTNode::new(ASTNodeType::Func(ASTFunc {
            ty, name, params, body
        })))
    }

    fn try_parse_var(&mut self) -> Option<ASTNode> {
        None
    }

    fn parse_current(&mut self) -> ASTNode {
        if self.current().ty == LexTokenType::EOF {
            return ASTNode::new(ASTNodeType::EOF);
        }

        self.set_checkpoint(self.pos);
        if let Some(node) = self.try_parse_var() {
            return node;
        }
        self.return_checkpoint();
        if let Some(node) = self.try_parse_func() {
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
        let root = parser.parse();

        if let ASTNodeType::Root(root) = root.ty {
            matches!(root.statements[0].ty, ASTNodeType::Func(_));
        } else {
            panic!("Not root!");
        }
    }

    #[test]
    fn simple_type() {
        let mut parser = Parser::new("unsigned long long int");
        let res = parser.try_parse_type().unwrap();

        assert_eq!(res.signed, false);
        assert_eq!(res.ty, ASTDataType::LongLong);

        let mut parser = Parser::new("signed");
        let res = parser.try_parse_type().unwrap();

        assert_eq!(res.signed, true);
        assert_eq!(res.ty, ASTDataType::Int);
    }

    #[test]
    fn simple_func_params() {
        let mut parser = Parser::new("(long long s, unsigned char a, unsigned c)");
        let res = parser.try_parse_func_params().unwrap();

        assert_eq!(res.len(), 3);
    }

}
