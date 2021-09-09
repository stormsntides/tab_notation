use std::fmt;

/// The literal type for guitar tab notation.
/// 
/// # Examples
/// 
/// ```
/// use data::Literal;
/// 
/// let num_lit = Literal::Number(4);
/// let op_lit = Literal::Options(String::from("time=4/4; fidelity=16"));
/// let no_lit = Literal::None;
/// ```
#[derive(Debug, PartialEq)]
pub enum Literal {
    /// A literal number.
    Number(u32),
    /// A literal string of options.
    Options(String),
    /// No literal.
    None,
}

/// The token type for guitar tab notation.
#[derive(Debug, PartialEq)]
pub enum TokenType {
    /* single character tokens */
    /// A single char representing a blank space: `.`
    Empty,
    /// A single char command that fills in the rest of the tab with empty chars: `,`
    Next,
    /* one or two character tokens */
    /// A single or two char representation of a note: `[A-G][b#]?`
    Note,
    /* multi character tokens */
    /// A multi-char representation of blank spaces: `:[0-9]+`
    SpreadEmpty,
    /// A multi-char representation of next commands: `;[0-9]+`
    SpreadNext,
    /* literals */
    /// A multi-char representation of a number: `[0-9]+`
    Number,
    /// A multi-char representation of option commands: `[time=4/4; fidelity=16]`
    Options,
    /* others */
    /// The end of the file.
    EndOfFile,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            TokenType::Empty => "Empty",
            TokenType::Next => "Next",
            TokenType::Note => "Note",
            TokenType::SpreadEmpty => "Spread Empty",
            TokenType::SpreadNext => "Spread Next",
            TokenType::Number => "Number",
            TokenType::Options => "Options",
            TokenType::EndOfFile => "EndOfFile",
        })
    }
}

/// Holds information relating to tokens generated from guitar tab files.
/// 
/// # Examples
/// 
/// ```
/// use data::{Token, TokenType, Literal};
/// 
/// let from_struct = Token {
///     type_of: TokenType::Number,
///     value: String::from("4"),
///     literal: Literal::Number(4),
///     line: 1
/// };
/// let from_new = Token::new(TokenType::Number, String::from("4"), Literal::Number(4), 1);
///
/// assert_eq!(from_struct, from_new);
///
/// let diff = Token::new(TokenType::Empty, String::from("."), Literal::None, 2);
///
/// assert_ne!(from_new, diff);
/// ```
#[derive(Debug, PartialEq)]
pub struct Token {
    /// The token's type.
    pub type_of: TokenType,
    /// The token's value; what is read from the file.
    pub value: String,
    /// The literal representation of the token's value. Could be `Literal::None` meaning a literal
    /// representation is not possible for this token.
    pub literal: Literal,
    /// The line the token was found on in the file.
    pub line: u32,
}

impl Token {
    /// Creates a new token.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use data::{Token, TokenType, Literal};
    /// 
    /// let number_token = Token::new(TokenType::Number, String::from("4"), Literal::Number(4), 1);
    /// let note_token = Token::new(TokenType::Note, String::from("A#"), Literal::None, 2);
    /// ```
    pub fn new(type_of: TokenType, value: String, literal: Literal, line: u32) -> Token {
        Token { type_of, value, literal, line }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{line_number}] {t_type} \"{value}\"", line_number=self.line, t_type=self.type_of, value=self.value)
    }
}

/// Struct for logging errors.
/// 
/// # Examples
/// 
/// ```
/// use data::Watcher;
/// 
/// let mut watcher = Watcher::new();
/// 
/// watcher.error(1, String::from("An error occurred here."));
/// watcher.error(5, String::from("This was an error."));
/// 
/// assert_eq!(
///     "[1] Error: An error occurred here.\n[5] Error: This was an error.",
///     watcher.to_string()
/// );
/// ```
pub struct Watcher {
    error_log: Vec<String>,
    pub had_error: bool,
}

impl Watcher {
    /// Creates a new watcher struct with default settings:
    /// 
    /// `error_log = vec![], had_error = false`
    pub fn new() -> Watcher {
        Watcher { error_log: vec![], had_error: false }
    }

    /// Logs an error; line is the line number the error occurred at, message is the error message
    /// to display to the user.
    pub fn error(&mut self, line: u32, message: String) {
        self.error_log.push(format!("[{}] Error: {}", line, message));
        self.had_error = true;
    }
}

impl fmt::Display for Watcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_log.join("\n"))
    }
}

#[cfg(test)]
mod data_tests {
    use super::*;

    #[test]
    fn compare_tokens() {
        let from_struct = Token {
            type_of: TokenType::Number,
            value: String::from("4"),
            literal: Literal::Number(4),
            line: 1
        };
        let from_new = Token::new(TokenType::Number, String::from("4"), Literal::Number(4), 1);
        
        assert_eq!(from_struct, from_new);

        let diff = Token::new(TokenType::Empty, String::from("."), Literal::None, 2);

        assert_ne!(from_new, diff);
    }
}
