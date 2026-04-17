#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTokenProps {
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexTokenType {
    Unsigned,
    Signed,
    Char,
    Short,
    Int,
    Float,
    Long,
    Double,
    Identifier,

    Numeral,
    String,

    Negation,
    Assign,
    Comma,
    Semi,

    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Asterisk,
    Div,
    Question,
    Colon,

    LParen,
    RParen,
    LSquare,
    RSquare,
    LBrace,
    RBrace,

    EOF,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexTokenProps {
    None,
    String(StringTokenProps),
    Numeral,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LexToken {
    pub ty: LexTokenType,
    pub start: usize, // inclusive
    pub end: usize, // exclusive
    pub literal: String,
    pub props: LexTokenProps,
}

impl LexToken {
    pub fn new(ty: LexTokenType, start: usize, end: usize, literal: &str) -> Self {
        Self {
            ty,
            start,
            end,
            literal: literal.into(),
            props: LexTokenProps::None,
        }
    }

    pub fn new_with_props(ty: LexTokenType, start: usize, end: usize, literal: &str, props: LexTokenProps) -> Self {
        Self {
            ty,
            start,
            end,
            literal: literal.into(),
            props,
        }
    }
}

pub struct Lexer {
    str: String,
    pos: usize,
}

impl Lexer {
    pub fn new(s: &str) -> Self {
        Self {
            str: s.into(),
            pos: 0,
        }
    }

    fn current_ch(&self) -> char {
        self.char_at(self.pos)
    }

    fn peek_ch(&self) -> char {
        self.char_at(self.pos + 1)
    }

    fn read_ch(&mut self) -> char {
        if self.pos >= self.str.len() {
            return '\0';
        }

        self.pos += 1;
        let ch = self.current_ch();

        ch
    }

    fn read_ch_offset(&mut self, offset: usize) {
        self.pos += offset;
    }

    fn get_literal(&self, start: usize, end: usize) -> &str {
        &self.str[start..end]
    }

    fn make_token(&self, ty: LexTokenType) -> LexToken {
        LexToken::new(ty, self.pos, self.pos + 1, &self.current_ch().to_string())
    }

    fn make_token_advance(&mut self, ty: LexTokenType) -> LexToken {
        let token = self.make_token(ty);
        self.read_ch();

        token
    }

    fn make_token_pos(&self, ty: LexTokenType, start: usize, end: usize, props: LexTokenProps) -> LexToken {
        LexToken::new_with_props(ty, start, end, self.get_literal(start, end), props)
    }

    fn make_doubled_advance(&mut self, ty: LexTokenType) -> LexToken {
        let token = self.make_token_pos(ty, self.pos, self.pos + 2, LexTokenProps::None);
        self.read_ch_offset(2);

        token
    }

    fn char_at(&self, pos: usize) -> char {
        if pos >= self.str.len() {
            return '\0';
        }

        let ch = self.str.chars().nth(pos).unwrap();

        ch
    }

    fn get_word_type(&self, start: usize, end: usize) -> LexTokenType {
        let literal = self.get_literal(start, end);

        match literal {
            "unsigned" => LexTokenType::Unsigned,
            "signed" => LexTokenType::Signed,
            "char" => LexTokenType::Char,
            "short" => LexTokenType::Short,
            "int" => LexTokenType::Int,
            "long" => LexTokenType::Long,
            "float" => LexTokenType::Float,
            "double" => LexTokenType::Double,
            _ => LexTokenType::Identifier,
        }
    }

    fn read_word(&mut self) -> LexToken {
        let start = self.pos;

        loop {
            let ch = self.read_ch();
            if !ch.is_alphabetic() {
                break;
            }
        }

        let end = self.pos;
        let ty = self.get_word_type(start, end);

        self.make_token_pos(ty, start, end, LexTokenProps::None)
    }

    fn read_numeral(&mut self) -> LexToken {
        let start = self.pos;

        loop {
            let ch = self.read_ch();
            if !ch.is_numeric() {
                break;
            }
        }

        let end = self.pos;
        self.make_token_pos(LexTokenType::Numeral, start, end, LexTokenProps::Numeral)
    }

    fn read_string(&mut self) -> LexToken {
        let start = self.pos;

        loop {
            let ch = self.read_ch();
            if ch == '"' {
                self.read_ch();
                break;
            } else if ch == '\0' {
                break;
            }
        }

        let end = self.pos;
        let val = self.get_literal(start+1, end-1);

        self.make_token_pos(LexTokenType::String, start, end, LexTokenProps::String(StringTokenProps { value: val.into() }))
    }

    fn read_plus(&mut self) -> LexToken {
        let peek = self.peek_ch();

        match peek {
            '+' => self.make_doubled_advance(LexTokenType::PlusPlus),
            _ => self.make_token_advance(LexTokenType::Plus),
        }
    }

    fn read_minus(&mut self) -> LexToken {
        let peek = self.peek_ch();

        match peek {
            '-' => self.make_doubled_advance(LexTokenType::MinusMinus),
            _ => self.make_token_advance(LexTokenType::Minus),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.current_ch().is_whitespace() {
            self.read_ch();
        }
    }

    fn next_token(&mut self) -> LexToken {
        self.skip_whitespace();

        let ch = self.current_ch();
        match ch {
            ch if ch.is_alphabetic() => self.read_word(),
            ch if ch.is_numeric() => self.read_numeral(),
            '"' => self.read_string(),
            '!' => self.make_token_advance(LexTokenType::Negation),
            '=' => self.make_token_advance(LexTokenType::Assign),
            ',' => self.make_token_advance(LexTokenType::Comma),
            ';' => self.make_token_advance(LexTokenType::Semi),
            '+' => self.read_plus(),
            '-' => self.read_minus(),
            '*' => self.make_token_advance(LexTokenType::Asterisk),
            '/' => self.make_token_advance(LexTokenType::Div),
            '?' => self.make_token_advance(LexTokenType::Question),
            ':' => self.make_token_advance(LexTokenType::Colon),
            '(' => self.make_token_advance(LexTokenType::LParen),
            ')' => self.make_token_advance(LexTokenType::RParen),
            '[' => self.make_token_advance(LexTokenType::LSquare),
            ']' => self.make_token_advance(LexTokenType::RSquare),
            '{' => self.make_token_advance(LexTokenType::LBrace),
            '}' => self.make_token_advance(LexTokenType::RBrace),
            '\0' => self.make_token_advance(LexTokenType::EOF),
            _ => self.make_token_advance(LexTokenType::Unknown),
        }
    }

    pub fn read(&mut self) -> Vec<LexToken> {
        let mut result = Vec::new();

        loop {
            let token = self.next_token();
            result.push(token.clone());

            if token.ty == LexTokenType::EOF {
                break;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::*;

    #[test]
    fn simple_words() {
        let mut lexer = Lexer::new("int main \"string test\"");
        let res = lexer.read();

        assert_eq!(res.len(), 4);
        assert_eq!(res[0].literal, "int");
        assert_eq!(res[0].ty, LexTokenType::Int);
        assert_eq!(res[1].literal, "main");
        assert_eq!(res[1].ty, LexTokenType::Identifier);
        assert_eq!(res[2].ty, LexTokenType::String);
        assert_eq!(res[2].props, LexTokenProps::String(StringTokenProps { value: "string test".into() }));
        assert_eq!(res[3].ty, LexTokenType::EOF);
    }

    #[test]
    fn simple_numerals() {
        //let mut lexer = Lexer::new("0x123");
    }

    #[test]
    fn simple() {
        let mut lexer = Lexer::new("int main() { int i = 5; }");
        let res = lexer.read();

        assert_eq!(res.len(), 12);
        assert_eq!(res[0].ty, LexTokenType::Int);
        assert_eq!(res[1].ty, LexTokenType::Identifier);
        assert_eq!(res[2].ty, LexTokenType::LParen);
        assert_eq!(res[3].ty, LexTokenType::RParen);
        assert_eq!(res[4].ty, LexTokenType::LBrace);
        assert_eq!(res[5].ty, LexTokenType::Int);
        assert_eq!(res[6].ty, LexTokenType::Identifier);
        assert_eq!(res[7].ty, LexTokenType::Assign);
        assert_eq!(res[8].ty, LexTokenType::Numeral);
        assert_eq!(res[9].ty, LexTokenType::Semi);
        assert_eq!(res[10].ty, LexTokenType::RBrace);
        assert_eq!(res[11].ty, LexTokenType::EOF);
    }

    #[test]
    fn simple_types() {
        let mut lexer = Lexer::new("unsigned long long int");
        let res = lexer.read();

        assert_eq!(res.len(), 5);
        assert_eq!(res[0].ty, LexTokenType::Unsigned);
        assert_eq!(res[1].ty, LexTokenType::Long);
        assert_eq!(res[2].ty, LexTokenType::Long);
        assert_eq!(res[3].ty, LexTokenType::Int);
    }

    #[test]
    fn simple_incr_decr() {
        let mut lexer = Lexer::new("+++----");
        let res = lexer.read();

        //assert_eq!(res.len(), 5);
        assert_eq!(res[0].ty, LexTokenType::PlusPlus);
        assert_eq!(res[1].ty, LexTokenType::Plus);
        assert_eq!(res[2].ty, LexTokenType::MinusMinus);
        assert_eq!(res[3].ty, LexTokenType::MinusMinus);
    }

}
