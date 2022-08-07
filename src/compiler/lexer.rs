use std::cell::Cell;

use anyhow::{anyhow, bail, Result};

static KEYWORDS: [&str; 11] = ["if", "then", "else", "elseif", "end", "endif", "while", "wend", "for", "match", "as"];

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
                } else if self.current_char().expect("oops") == '<' {
                    Token::PartLessLess
                } else {
                    Token::SymLess
                }
            },
            '>' => {
                self.advance();
                if self.current_char().expect("oops") == '=' {
                    self.advance();
                    Token::PartMoreEq
                } else if self.current_char().expect("oops") == '>' {
                    Token::PartMoreMore
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
            '"' => {
                self.advance();
                let scanned_content = self.scan_string_literal().unwrap();
                assert_eq!(self.consume_char().unwrap(), '"', "The string literal is not terminated");
                Token::StringLiteral {
                    content: scanned_content,
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
                        "as" => Token::KeywordAs,
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

    fn scan_string_literal(&self) -> Result<String> {
        self.advance();
        let mut buf = String::new();
        let mut in_escape = false;
        loop {
            if in_escape {
                match self.current_char().expect("oops") {
                    '"' => {
                        buf.push('"');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    '\\' => {
                        buf.push('\\');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    'a' => {
                        buf.push('\x07');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    'b' => {
                        buf.push('\x08');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    't' => {
                        buf.push('\t');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    'n' => {
                        buf.push('\n');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    'v' => {
                        buf.push('\x0b');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    'f' => {
                        buf.push('\x0c');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    'r' => {
                        buf.push('\x0d');
                        self.consume_char()?;
                        in_escape = false;
                        continue
                    }
                    'u' => {
                        self.consume_char()?;
                        // TODO: support unicode codepoint sequence: \u{ 3015 3028 3033 }
                        let mut codepoint: u16 = 0;
                        for _ in 0..=3 {
                            let or_value = match self.current_char()
                                .expect("There must be more characters to recognize Unicode escape sequence") {
                                '0' => 0,
                                '1' => 1,
                                '2' => 2,
                                '3' => 3,
                                '4' => 4,
                                '5' => 5,
                                '6' => 6,
                                '7' => 7,
                                '8' => 8,
                                '9' => 9,
                                'A' | 'a' => 0xA,
                                'B' | 'b' => 0xB,
                                'C' | 'c' => 0xC,
                                'D' | 'd' => 0xD,
                                'E' | 'e' => 0xE,
                                'F' | 'f' => 0xF,
                                '"' => bail!("An Unicode escape sequence must have four hexadecimal codepoint, but there's no codepoint anymore."),
                                other_char => bail!("An Unicode escape sequence must have four hexadecimal codepoint, but there's other character that is not valid a codepoint character."),
                            };
                            codepoint = (codepoint << 4) | or_value;
                            self.consume_char().unwrap();
                        }
                        let surrogate_code_point_range = 0xD800..=0xDFFF;
                        if surrogate_code_point_range.contains(&codepoint) {
                            bail!("This codepoint ({codepoint:x}) is invalid Unicode scalar value. The codepoint of unicode escape sequence must not be a surrogate code points.\
                            Note: An \"Unicode scalar value\" is defined as \"any Unicode code point except high-surrogate and low-surrogate code points.\" in the Unicode glossary.\
                                  To see full definition, please see https://www.unicode.org/glossary/#unicode_scalar_value")
                        }
                        buf.push(char::from_u32(codepoint as u32).unwrap());
                        in_escape = false;
                        continue
                    }
                    other_char => {
                        bail!("The char ({other_char}) is not an acceptable char as escape-sequence.")
                    }
                }
            } else {
                match self.current_char().expect("oops") {
                    '"' => break,
                    '\\' => {
                        in_escape = true;
                        continue
                    }
                    '\n' => bail!("String literal can not contain newline literally. To script newline, please escape as \"\\n\"."),
                    other_char => buf.push(other_char),
                }
            }
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
    StringLiteral {
        content: String,
    },
    EndOfFile,
    /// `"\n"`
    NewLine,
    /// `"var"`
    VarKeyword,
    KeywordTrue,
    KeywordFalse,
    KeywprdAs,
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
    /// `>>`
    PartMoreMore,
    /// `<`
    SymLess,
    /// `<<`
    PartLessLess,
    /// `&`
    SymAnd,
    /// `&&`
    PartAndAnd,
    /// `^`
    SymCaret,
    /// `|`
    SymPipe,
    /// `||`
    PartPipePipe,
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