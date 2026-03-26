#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexTokenType {
    Keyword,
    Identifier,
    EOF,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LexToken {
    pub ty: LexTokenType,
    pub start: usize, // inclusive
    pub end: usize, // exclusive
    pub literal: String,
}

impl LexToken {
    pub fn new(ty: LexTokenType, start: usize, end: usize, literal: &str) -> Self {
        Self {
            ty,
            start,
            end,
            literal: literal.into(),
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

    fn get_literal(&self, start: usize, end: usize) -> &str {
        &self.str[start..end]
    }

    fn make_token(&self, ty: LexTokenType) -> LexToken {
        LexToken::new(ty, self.pos, self.pos + 1, &self.current_ch().to_string())
    }

    fn make_token_pos(&self, ty: LexTokenType, start: usize, end: usize) -> LexToken {
        LexToken::new(ty, start, end, self.get_literal(start, end))
    }

    fn current_ch(&self) -> char {
        if self.pos >= self.str.len() {
            return '\0';
        }

        let ch = self.str.chars().nth(self.pos).unwrap();

        ch
    }

    fn read_ch(&mut self) -> char {
        if self.pos >= self.str.len() {
            return '\0';
        }

        self.pos += 1;
        let ch = self.current_ch();

        ch
    }

    fn get_word_type(&self, start: usize, end: usize) -> LexTokenType {
        let literal = self.get_literal(start, end);

        match literal {
            "int" => LexTokenType::Keyword,
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
        self.make_token_pos(ty, start, end)
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
            '\0' => self.make_token(LexTokenType::EOF),
            _ => self.make_token(LexTokenType::Unknown),
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
        let mut lexer = Lexer::new("int main");
        let res = lexer.read();

        assert_eq!(res.len(), 3);
        assert_eq!(res[0].literal, "int");
        assert_eq!(res[0].ty, LexTokenType::Keyword);
        assert_eq!(res[1].literal, "main");
        assert_eq!(res[1].ty, LexTokenType::Identifier);
        assert_eq!(res[2].ty, LexTokenType::EOF);
    }

}
