use crate::lexer::Token;

// AST fill follow the BNF from the stasndard for now.
pub type Span = (usize, usize);
#[derive(Debug)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub enum Instruction {
    Label(Token),

    // Kayword Instructions
    Say,
    Signal,
    
    Unknown(Token),
}

