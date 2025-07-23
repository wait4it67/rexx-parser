use crate::lexer::types::{Range, Token, TokenType};

use super::{types::LogicalLine, Position};

pub struct Lexer<'a> {
    line_counter: usize,
    line_start_index: usize,
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        Lexer {
            source: src,
            line_counter: 0,
            line_start_index: 0,
        }
    }

    pub fn get_text(&self, token: &Token) -> &str {
        &self.source[token.range.start.index..token.range.end.index]
    }

    pub fn tokenize(&mut self) -> Vec<LogicalLine> {
        let mut logical_lines = Vec::new();
        let mut line = LogicalLine { tokens: Vec::new() };

        let mut chars = self.source.char_indices().peekable();
        while let Some((pos, ch)) = chars.next() {
            let mut eol = false;
            let token = match ch {
                ' ' | '\t' => self.consume_whitespaces(&mut chars, pos, &ch),
                '\n' | '\r' => {
                    let after_comma = line.tokens.last().is_some()
                        && line.tokens.last().unwrap().token_type == TokenType::Comma;
                    let before_eos = chars.peek().is_none();
                    eol = if !after_comma {
                        true
                    } else if !before_eos {
                        false
                    } else {
                        true
                    };
                    let line_token =Token {
                        token_type: TokenType::EOL,
                        range: self.make_one_line_range(pos, pos + 1),
                    };
                    self.line_counter += 1;
                    self.line_start_index = pos + 1;
                    line_token
                }
                '/' => {
                    if let Some((_, '*')) = chars.peek() {
                        self.consume_block_comment(&mut chars, pos)
                    } else {
                        Token {
                            token_type: TokenType::Todo,
                            range: self.make_one_line_range(pos, pos + 1),
                        }
                    }
                }
                '"' | '\'' => self.consume_string_literal(&mut chars, pos, ch),
                ',' => Token {
                    token_type: TokenType::Comma,
                    range: self.make_one_line_range(pos, pos + 1),
                },
                ':' => Token {
                    token_type: TokenType::Colon,
                    range: self.make_one_line_range(pos, pos + 1),
                },
                // TODO: arephmetics
                '+' | '-' | '*' | '(' | ')' => Token {
                    token_type: TokenType::Todo,
                    range: self.make_one_line_range(pos, pos + 1),
                },
                '=' => Token {
                    token_type: TokenType::Semicolon,
                    range: self.make_one_line_range(pos, pos + 1),
                },
                ';' => {
                    eol = true;
                    Token {
                        token_type: TokenType::Semicolon,
                        range: self.make_one_line_range(pos, pos + 1),
                    }
                }

                c if c.is_ascii_alphabetic() => self.consume_identifier(&mut chars, pos),
                c if c.is_ascii_digit() => self.consume_number(&mut chars, pos),

                _ => Token {
                    token_type: TokenType::Unknown,
                    range: self.make_one_line_range(pos, pos + 1),
                },
            };
            line.tokens.push(token);

            if eol {
                logical_lines.push(line);
                line = LogicalLine { tokens: Vec::new() };
            }
        }
        line.tokens.push(Token {
            token_type: TokenType::EOS,
            range: self.make_one_line_range(self.source.len(), self.source.len()),
        });
        logical_lines.push(line);
        logical_lines
    }

    fn consume_identifier(
        &self,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        start: usize,
    ) -> Token {
        let mut end = start + 1;
        while let Some((_, ch)) = chars.peek() {
            if !ch.is_ascii_alphanumeric() && *ch != '_' {
                break;
            }
            end += 1;
            chars.next();
        }
        Token {
            token_type: TokenType::Identifier,
            range: self.make_one_line_range(start, end),
        }
    }

    fn consume_number(
        &self,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        start: usize,
    ) -> Token {
        let mut end = start + 1;
        while let Some((_, ch)) = chars.peek() {
            if !ch.is_ascii_digit() {
                break;
            }
            end += 1;
            chars.next();
        }
        Token {
            token_type: TokenType::Number,
            range: self.make_one_line_range(start, end),
        }
    }

    fn consume_string_literal(
        &self,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        start: usize,
        first_quote: char,
    ) -> Token {
        let mut end = start + 1;
        while let Some((_, ch)) = chars.next() {
            end += 1;
            if ch == first_quote {
                break;
            } else if ch == '\\' {
                // TODO: diagnostics on unsupported escape sequences
                chars.next();
                end += 1;
            }
        }
        Token {
            token_type: TokenType::Literal,
            range: self.make_one_line_range(start, end),
        }
    }

    fn consume_whitespaces(
        &self,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        start: usize,
        initial_ch: &char,
    ) -> Token {
        let mut end = start + 1;
        while let Some((_, ch)) = chars.peek() {
            if ch != initial_ch {
                break;
            }
            end += 1;
            chars.next();
        }
                Token {
            token_type: TokenType::Whitespace,
            range: self.make_one_line_range(start, end),
        }
    }

    fn consume_block_comment(
        &mut self,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        start: usize,
    ) -> Token {
        let start_pos = self.make_position(start);
        let mut depth = 0;
        let mut end = start + 1;
        chars.next(); // Consume '*' after '/'
        end += 1;
        while let Some((pos, ch)) = chars.next() {
            end += 1;
            if ch == '\n' || ch == '\r' {
                self.line_counter += 1;
                self.line_start_index = pos + 1;
            }
            if ch == '*' {
                if let Some((_, '/')) = chars.peek() {
                    chars.next();
                    end += 1;
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
            }
            if ch == '/' {
                if let Some((_, '*')) = chars.peek() {
                    chars.next();
                    end += 1;
                    depth += 1;
                }
            }
        }
        let range = Range {
            start: start_pos,
            end: self.make_position(end),
        };
        Token {
            token_type: TokenType::Comment,
            range,
        }
    }

    fn make_position(&self, index: usize) -> Position {
        Position {
            line: self.line_counter,
            character: index - self.line_start_index,
            index,
        }
    }
    fn make_one_line_range(&self, start_idx: usize, end_index: usize) -> Range {
        Range {
            start: self.make_position(start_idx),
            end: self.make_position(end_index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn lex_say() {
    //     let lexer = Lexer::new("say");
    //     let result = lexer.tokenize();
    //     assert_eq!(result.len(), 1);
    //     assert_eq!(Token {token_type: TokenType::General, lexeme_span: (0, 3)}, *result.get(0).unwrap().tokens.get(0).unwrap());
    // }

    #[test]
    fn lex_comment1() {
        let mut lexer = Lexer::new("/* This is a comment */");
        let result = lexer.tokenize();
        let token = result.get(0).unwrap().tokens.get(0).unwrap();
        assert_eq!(result.get(0).unwrap().tokens.len(), 2); // Comment and eos
        assert_eq!("/* This is a comment */", lexer.get_text(token));
    }
    #[test]
    fn lex_comment2() {
        let mut lexer = Lexer::new("/* This /* is a */ comment */");
        let result = lexer.tokenize();
        assert_eq!(result.get(0).unwrap().tokens.len(), 2); // Comment and eos
    }
    #[test]
    fn lex_lines() {
        let mut lexer = Lexer::new("SAY 'a'; SAY 'b'");
        let result = lexer.tokenize();
        assert_eq!(result.len(), 2);
        assert_eq!(result.len(), 2);
    }
    #[test]
    fn lex_lines2() {
        let mut lexer = Lexer::new("SAY 'a',\n'b'");
        let result = lexer.tokenize();
        assert_eq!(result.len(), 1);
        let eol_token = result[0].tokens[4].clone();
        let b_token = result[0].tokens[5].clone();
        assert_eq!(eol_token.range.start.line, 0);
        assert_eq!(b_token.range.start.line, 1);
        assert_eq!(b_token.range.start.character, 0);
    }
    #[test]
    fn lex_lines3() {
        let mut lexer = Lexer::new("SAY 'a',\n");
        let result = lexer.tokenize();
        assert_eq!(result.len(), 2);
    }
    #[test]
    fn lex_line4() {
        let mut lexer = Lexer::new("/*/\n*/\nloop:");
        let result = lexer.tokenize();
        assert_eq!(result[0].tokens[0].range.end.line, 1);
        assert_eq!(result[0].tokens[0].range.end.character, 2);
    }
    #[test]
    fn lex_line5() {
        let mut lexer = Lexer::new("/*\n *\n */\nx:");
        let result = lexer.tokenize();
        assert_eq!(result[0].tokens[0].range.end.line, 2);
        assert_eq!(result[0].tokens[0].range.end.character, 3);
    }

    // TODO http://www.manmrk.net/tutorials/rexx/rexxvmref.pdf page 29
}
