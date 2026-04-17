use std::vec;

use crate::lexer::*;

#[derive(Debug, Clone)]
pub struct ASTRoot {
    pub statements: Vec<ASTNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ASTIdentifier {
    pub literal: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ASTUnaryType {
    Negation,
    Plus,
    Minus,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ASTUnary {
    pub ty: ASTUnaryType,
    pub expr: Box<ASTExpression>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ASTBinaryType {
    Add,
    Sub,
    Mult,
    Div,
    Indexing,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ASTBinary {
    pub ty: ASTBinaryType,
    pub left: Box<ASTExpression>,
    pub right: Box<ASTExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ASTConditional {
    pub cond: Box<ASTExpression>,
    pub then: Box<ASTExpression>,
    pub or: Box<ASTExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ASTExpression {
    Null,
    Identifier(ASTIdentifier),
    Unary(ASTUnary),
    Binary(ASTBinary),
    Conditional(ASTConditional),
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

#[derive(Debug, Clone)]
pub struct ASTVar {
    pub ty: ASTData,
    pub name: ASTIdentifier,
    pub initializer: Option<ASTExpression>,
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

    fn prefix_binding_power(&self, ty: ASTUnaryType) -> u8 {
        match ty {
            ASTUnaryType::Plus | ASTUnaryType::Minus | ASTUnaryType::Negation
            | ASTUnaryType::PreDec | ASTUnaryType::PreInc => 12,
            _ => 0,
        }
    }

    fn parse_unary(&mut self, ty: ASTUnaryType) -> Option<ASTExpression> {
        self.read();
        let expr = self.parse_expression(self.prefix_binding_power(ty))?;

        Some(ASTExpression::Unary(ASTUnary { ty, expr: Box::new(expr) }))
    }

    fn parse_prefix(&mut self) -> Option<ASTExpression> {
        let current = self.current();
        let res = match current.ty {
            LexTokenType::Identifier | LexTokenType::Numeral => Some(ASTExpression::Identifier(ASTIdentifier { literal: current.literal.clone() })),
            LexTokenType::Negation => self.parse_unary(ASTUnaryType::Negation),
            LexTokenType::Plus => self.parse_unary(ASTUnaryType::Plus),
            LexTokenType::Minus => self.parse_unary(ASTUnaryType::Minus),
            LexTokenType::PlusPlus => self.parse_unary(ASTUnaryType::PreInc),
            LexTokenType::MinusMinus => self.parse_unary(ASTUnaryType::PreDec),
            _ => None,
        };

        if res.is_some() {
            self.read();
        }

        res
    }

    fn infix_binding_power(&self, ty: LexTokenType) -> (u8, u8) {
        match ty {
            LexTokenType::Plus | LexTokenType::Minus => (5, 6),
            LexTokenType::Asterisk | LexTokenType::Div => (7, 8),
            LexTokenType::Assign => (2, 1),
            LexTokenType::LSquare => (13, 13),
            LexTokenType::Question => (4, 3),
            _ => (0, 0),
        }
    }

    fn parse_binary_expression(&mut self, lhs: ASTExpression, min_bp: u8, ty: ASTBinaryType) -> Option<ASTExpression> {
        self.read();
        let rhs = self.parse_expression(min_bp)?;

        Some(ASTExpression::Binary(ASTBinary { ty, left: Box::new(lhs), right: Box::new(rhs) }))
    }

    fn parse_indexing_expression(&mut self, lhs: ASTExpression, min_bp: u8) -> Option<ASTExpression> {
        self.read();
        let rhs = self.parse_expression(min_bp)?;

        self.try_parse_current(LexTokenType::RSquare)?;

        Some(ASTExpression::Binary(ASTBinary { ty: ASTBinaryType::Indexing, left: Box::new(lhs), right: Box::new(rhs) }))
    }

    fn parse_conditional_expression(&mut self, lhs: ASTExpression, min_bp: u8) -> Option<ASTExpression> {
        self.read();

        let then = self.parse_expression(min_bp)?;
        self.try_parse_current(LexTokenType::Colon)?;

        let or = self.parse_expression(min_bp)?;

        Some(ASTExpression::Conditional(ASTConditional { cond: Box::new(lhs), then: Box::new(then), or: Box::new(or) }))
    }

    fn parse_infix_expression(&mut self, lhs: ASTExpression, min_bp: u8, ty: LexTokenType) -> Option<ASTExpression> {
        match ty {
            LexTokenType::Plus => self.parse_binary_expression(lhs, min_bp, ASTBinaryType::Add),
            LexTokenType::Minus => self.parse_binary_expression(lhs, min_bp, ASTBinaryType::Sub),
            LexTokenType::Asterisk => self.parse_binary_expression(lhs, min_bp, ASTBinaryType::Mult),
            LexTokenType::Div => self.parse_binary_expression(lhs, min_bp, ASTBinaryType::Div),
            LexTokenType::LSquare => self.parse_indexing_expression(lhs, min_bp),
            LexTokenType::Question => self.parse_conditional_expression(lhs, min_bp),
            _ => Some(ASTExpression::Null),
        }
    }

    fn parse_expression(&mut self, min_bp: u8) -> Option<ASTExpression> {
        let mut lhs = self.parse_prefix()?;

        loop {
            let current = self.current();
            let (lbp, rbp) = self.infix_binding_power(current.ty);
            if lbp < min_bp {
                break;
            }

            let temp = self.parse_infix_expression(lhs.clone(), rbp, current.ty)?;
            if temp == ASTExpression::Null {
                break;
            }

            lhs = temp;
        }

        Some(lhs)
    }

    fn get_default_signedness(&self, _: ASTDataType) -> bool {
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

    fn try_parse_block(&mut self) -> Option<Vec<ASTNode>> {
        let _ = self.try_parse_current(LexTokenType::LBrace)?;
        // let body = ;
        let _ = self.try_parse_current(LexTokenType::RBrace)?;

        None
    }

    fn try_parse_func(&mut self) -> Option<ASTNode> {
        let ty = self.try_parse_type()?;
        let name = self.try_parse_identifier()?;
        let params = self.try_parse_func_params()?;
        //let body = self.try_parse_block()?;
        let body = vec![];

        Some(ASTNode::new(ASTNodeType::Func(ASTFunc {
            ty, name, params, body
        })))
    }

    fn try_parse_var(&mut self) -> Option<ASTNode> {
        let ty = self.try_parse_type()?;
        let name = self.try_parse_identifier()?;

        if self.try_parse_current(LexTokenType::Semi).is_some() {
            return Some(ASTNode::new(ASTNodeType::Var(ASTVar {
                ty, name, initializer: None
            })))
        }

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
            assert!(matches!(root.statements[0].ty, ASTNodeType::Func(_)));
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

    #[test]
    fn simple_expr_addmult() {
        let mut parser = Parser::new("x + y * c");
        let res = parser.parse_expression(0).unwrap();

        if let ASTExpression::Binary(binary) = res {
            assert_eq!(binary.ty, ASTBinaryType::Add);
        } else {
            panic!("Not a binary expression");
        }

        let mut parser = Parser::new("x * y + c");
        let res = parser.parse_expression(0).unwrap();

        if let ASTExpression::Binary(binary) = res {
            assert_eq!(binary.ty, ASTBinaryType::Add);
        } else {
            panic!("Not a binary expression");
        }
    }

    #[test]
    fn simple_test_prepostfix() {
        let mut parser = Parser::new("-2[x]");
        let res = parser.parse_expression(0).unwrap();

        if let ASTExpression::Unary(unary) = res {
            assert_eq!(unary.ty, ASTUnaryType::Minus);

            if let ASTExpression::Binary(inner) = *unary.expr {
                assert_eq!(inner.ty, ASTBinaryType::Indexing);
            } else {
                panic!("Not an inner unary expression");
            }
        } else {
            panic!("Not a unary expression");
        }
    }

    #[test]
    fn simple_test_conditional() {
        let mut parser = Parser::new("c * a ? o + 1 ? 2 : 1 : 3");
        let res = parser.parse_expression(0).unwrap();

        if let ASTExpression::Conditional(cond) = res {
            if let ASTExpression::Conditional(inner) = *cond.then {
                assert!(matches!(*inner.cond, ASTExpression::Binary(_)));
            } else {
                panic!("Not an inner conditional expression");
            }
        } else {
            panic!("Not a conditional expression");
        }
    }

}
