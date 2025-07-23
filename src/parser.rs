use crate::ast::{Instruction, Program};
use crate::lexer::{Lexer, LogicalLine, Token, TokenType};

#[derive(Debug, PartialEq)]
pub enum ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;

// #[derive(Debug, PartialEq)]
// struct Node {}

pub struct RexxParser<'a> {
    lexer: &'a mut Lexer<'a>,
}

impl<'a> RexxParser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> RexxParser<'a> {
        RexxParser { lexer }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut program = Program {
            instructions: vec![],
        };
        for line in self.lexer.tokenize() {
            self.parse_line(&line, &mut program);
            // program.instructions.push();
        }
        Ok(program)
    }
    fn parse_line(&self, line: &LogicalLine, program: &mut Program) {
        let tokens = line
            .tokens
            .iter()
            .filter(|t| {
                t.token_type != TokenType::Whitespace
                    && t.token_type != TokenType::Comment
                    && t.token_type != TokenType::EOL
                    && t.token_type != TokenType::EOS
            })
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            return;
        }

        match tokens[0] {
            Token {
                token_type: TokenType::Unknown,
                ..
            } => program
                .instructions
                .push(Instruction::Unknown(tokens[0].clone())),
            Token {
                token_type: TokenType::Identifier,
                ..
            } => {
                if Self::is_label(&tokens) {
                    program
                        .instructions
                        .push(Instruction::Label((*tokens[0]).clone()));
                    return;
                }
                if Self::is_assignment(&tokens) {
                    return;
                }
                if self.is_kayword_instruction(&tokens) {
                    self.parse_kayword_instruction(&tokens, program);
                    return;
                }
            }
            _ => eprintln!("Unexpected token: {:?}", tokens[0]),
        }
        // for token in tokens {
        //     println!("{:?}: {}", token, self.lexer.get_text(token));
        // }
    }

    fn is_label(tokens: &Vec<&Token>) -> bool {
        if tokens.len() > 1 {
            tokens.get(0).unwrap().token_type == TokenType::Identifier
                && tokens.get(1).unwrap().token_type == TokenType::Colon
        } else {
            false
        }
    }
    fn is_assignment(tokens: &Vec<&Token>) -> bool {
        if tokens.len() > 1 {
            tokens.get(0).unwrap().token_type == TokenType::Identifier
                && tokens.get(1).unwrap().token_type == TokenType::Equal
        } else {
            false
        }
    }
    fn is_kayword_instruction(&self, tokens: &Vec<&Token>) -> bool {
        match self
            .lexer
            .get_text(tokens.get(0).unwrap())
            .to_uppercase()
            .as_str()
        {
            "ADDRESS" | "ARG" | "CALL" | "DROP" | "EXIT" | "INTERPRET" | "ITERATE" | "LEAVE"
            | "NOP" | "NUMERIC" | "OPTIONS" | "PARSE" | "PROCEDURE" | "PULL" | "PUSH" | "QUEUE"
            | "RETURN" | "SAY" | "SIGNAL" | "TRACE" | "THEN" | "ELSE" | "WHEN" | "OTHERWISE" => {
                return true
            }
            _ => false,
        }
    }
    fn parse_kayword_instruction(&self, tokens: &Vec<&Token>, program: &mut Program) {
        program.instructions.push(
            match self
                .lexer
                .get_text(tokens.get(0).unwrap())
                .to_uppercase()
                .as_str()
            {
                // "ADDRESS" | "ARG" | "CALL" | "DROP" | "EXIT" | "INTERPRET" | "ITERATE" | "LEAVE"
                // | "NOP" | "NUMERIC" | "OPTIONS" | "PARSE" | "PROCEDURE" | "PULL" | "PUSH" | "QUEUE"
                // | "RETURN" |
                "SAY" => Instruction::Say,
                "SIGNAL" => Instruction::Signal,
                // | "TRACE" | "THEN" | "ELSE" | "WHEN" | "OTHERWISE" => Instruction::Keyword(tokens[0].lexeme_span),
                _ => Instruction::Unknown(tokens[0].clone()),
            },
        );
    }

    pub fn get_text(&self, token: &Token) -> &str {
        &self.lexer.get_text(token)
    }

}

/*
keyword_instruction :=
    address |
    arg |
    call |
    drop |
    exit |
    interpret |
    iterate |
    leave |
    nop |
    numeric |
    options |
    parse |
    procedure |
    pull |
    push |
    queue |
    return |
    say |
    signal |
    trace |
    'THEN' Msg8.1 |
    'ELSE' Msg8.2 |
    'WHEN' Msg9.1 |
    'OTHERWISE' Msg9.2
*/

// // #[test]
// // fn parse_say() {
// //     let lexer = Lexer::new("say");
// //     let parser = RexxParser::new(&lexer);
// //     let result = parser.parse("say").unwrap();
// //     assert_eq!(result, Node {});
// // }
