use std::cell::Cell;

use anyhow::{anyhow, Result};

static KEYWORDS: [&str; 10] = ["if", "then", "else", "elseif", "end", "endif", "while", "wend", "for", "match"];
pub struct Lexer {
    index: Cell<usize>,
    current_source: String,
}

impl Lexer {
    pub fn create(source: &str) -> Self {
        Self {
            current_source: source.to_string(),
            index: Cell::new(0),
        }
    }

    fn drain_space(&self) {
        while !self.reached_end() && self.current_char().expect("oops") == ' ' {
            self.index.set(self.index.get() + 1);
        }
    }

    pub fn next(&self) -> Token {
        self.drain_space();

        if self.reached_end() {
            return Token::EndOfFile
        }

        let c = self.current_char().expect("oops");
        match c {
            '\n' => {
                self.advance();
                Token::NewLine
            },
            '=' => {
                self.advance();
                if self.current_char().expect("oops") == '=' {
                    self.advance();
                    Token::PartEqEq
                } else {
                    Token::SymEq
                }
            },
            '+' => {
                self.advance();
                Token::SymPlus
            },
            '-' => {
                self.advance();
                Token::SymMinus
            },
            '*' => {
                self.advance();
                Token::SymAsterisk
            },
            '/' => {
                self.advance();
                if self.current_char().expect("oops") == '/' {
                    self.advance();
                    Token::Comment {
                        content: self.scan_comment_content().expect("failed to process")
                    }
                } else {
                    Token::SymSlash
                }
            },
            '(' => {
                self.advance();
                Token::SymLeftPar
            },
            ')' => {
                self.advance();
                Token::SymRightPar
            },
            '#' => {
                self.advance();
                Token::SymSharp
            },
            '[' => {
                self.advance();
                Token::SymOpenBracket
            },
            ']' => {
                self.advance();
                Token::SymCloseBracket
            },
            ':' => {
                self.advance();
                Token::SymColon
            },
            '.' => {
                self.advance();
                Token::SymDot
            },
            '<' => {
                self.advance();
                if self.current_char().expect("oops") == '=' {
                    self.advance();
                    if self.current_char().expect("oops") == '>' {
                        self.advance();
                        Token::PartLessEqMore
                    } else {
                        Token::PartLessEq
                    }
                } else {
                    Token::SymLess
                }
            },
            '>' => {
                self.advance();
                if self.current_char().expect("oops") == '=' {
                    self.advance();
                    Token::PartMoreEq
                } else {
                    Token::SymMore
                }
            },
            '!' => {
                self.advance();
                if self.current_char().expect("oops") == '=' {
                    self.advance();
                    Token::PartBangEq
                } else {
                    Token::SymBang
                }
            },
            c if c.is_ascii_digit() => self.scan_digits().expect("oops"),
            c if c.is_ascii_lowercase() => {
                let scan_result = self.scan_lowers().expect("oops");
                let is_keyword = KEYWORDS.contains(&scan_result.as_str());
                if is_keyword {
                    match scan_result.as_str() {
                        "var" => Token::VarKeyword,
                        "true" => Token::KeywordTrue,
                        "false" => Token::KeywordFalse,
                        other => Token::Reserved {
                            matched: other.to_string(),
                        }
                    }
                } else {
                    Token::Identifier { inner: scan_result }
                }
            },
            other => Token::UnexpectedChar {
                index: self.index.get(),
                char: other,
            }
        }
    }

    fn scan_digits(&self) -> Result<Token> {
        let mut buf = String::new();
        loop {
            if self.reached_end() {
                break
            }

            // DON'T CONSUME!!
            let c = self.current_char()?;
            if c.is_ascii_digit() {
                break
            }
            let c = self.consume_char()?;

            buf.push(c);
        }

        Ok(Token::Digits {
            sequence: buf
        })
    }

    fn scan_lowers(&self) -> Result<String> {
        let mut buf = String::new();
        loop {
            if self.reached_end() {
                break
            }

            // DON'T CONSUME!!
            let c = self.current_char()?;
            if c.is_ascii_lowercase() {
                break
            }
            let c = self.consume_char()?;

            buf.push(c);
        }

        Ok(buf)
    }

    fn scan_comment_content(&self) -> Result<String> {
        let mut buf = String::new();
        while !self.reached_end() && self.current_char().expect("oops") != '\n' {
            buf.push(self.consume_char()?)
        }
        Ok(buf)
    }

    pub fn peek(&self) -> Token {
        let current_index = self.index.get();
        let token = self.next();
        self.index.set(current_index);
        token
    }

    fn current_char(&self) -> Result<char> {
        self.current_source
            .as_str()
            .chars()
            .nth(self.index.get())
            .ok_or_else(||
                anyhow!("index: out of range (idx={request}, max={max})",
                    request = self.index.get(),
                    max = self.current_source.len()
                )
            )
    }

    fn consume_char(&self) -> Result<char> {
        let c = self.current_char()?;
        self.advance();
        Ok(c)
    }

    fn reached_end(&self) -> bool {
        self.index.get() >= self.current_source.len()
    }

    fn advance(&self) {
        self.advance_by(1);
    }

    fn advance_by(&self, step: usize) {
        self.index.set(self.index.get() + step);
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Identifier {
        inner: String,
    },
    Digits {
        sequence: String,
    },
    UnexpectedChar {
        index: usize,
        char: char,
    },
    Comment {
        content: String,
    },
    EndOfFile,
    /// `"\n"`
    NewLine,
    /// `"var"`
    VarKeyword,
    KeywordTrue,
    KeywordFalse,
    /// `"="`
    SymEq,
    /// `"+"`
    SymPlus,
    /// `"-"`
    SymMinus,
    /// `*`
    SymAsterisk,
    /// `/`
    SymSlash,
    /// `"("`
    SymLeftPar,
    /// `")"`
    SymRightPar,
    /// `>`
    SymMore,
    /// `<`
    SymLess,
    /// `!`
    SymBang,
    /// `==`
    PartEqEq,
    /// `!=`
    PartBangEq,
    /// `<=`
    PartLessEq,
    /// `>=`
    PartMoreEq,
    /// `<=>`
    PartLessEqMore,
    /// `#`
    SymSharp,
    /// `[`
    SymOpenBracket,
    /// `]`
    SymCloseBracket,
    /// `:`
    SymColon,
    /// `.`
    SymDot,
    /// reserved for future use.
    Reserved {
        matched: String,
    },

}