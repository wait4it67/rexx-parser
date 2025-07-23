use crate::ast::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Whitespace,
    Comment,
    Literal,
    Number,
    Comma,
    Colon,
    Identifier,
    Semicolon,
    Equal,
    Todo,
    Unknown,
    EOL, // the end of the line
    EOS, // the end of the source
}

#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub line: usize,
    pub character: usize,
    pub index: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub range: Range,
}

#[derive(Debug, PartialEq)]
pub struct LogicalLine {
    pub tokens: Vec<Token>,
}
