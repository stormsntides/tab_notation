use data::{Token, TokenType, Literal, Watcher};

/// Keeps track of the position within a string of text contained in a `Lexer` struct.
struct Cursor {
    start: u32,
    current: u32,
    line: u32,
    length: usize,
}

impl Cursor {
    /// Create a new `Cursor` of size `length` with default starting values for the following properties:
    /// 
    /// `start = 0, current = 0, line = 1`
    fn new(length: usize) -> Cursor {
        Cursor { start: 0, current: 0, line: 1, length }
    }

    /// Returns true if this `Cursor` is at the end of the provided string length.
    fn is_at_end(&self) -> bool {
        self.current >= self.length as u32
    }
}

/// The lexical analyzer struct for generating tokens from a source string.
/// 
/// # Examples
/// 
/// ```
/// use lexer::Lexer;
/// use data::{Token, TokenType, Literal};
/// 
/// let mut lexer = Lexer::new("[time=4/4; fidelity=16] E A D G B E\n. 2 7,\n:2 5 ;4 1".to_string());
/// let tokens_or_error: Result<&Vec<Token>, String> = lexer.generate_tokens();
/// ```
pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    cursor: Cursor,
    watcher: Watcher,
}

impl Lexer {
    /// Creates a new Lexer struct using the provided string as its source.
    pub fn new(source: String) -> Lexer {
        let length = source.len();
        Lexer { source, tokens: Vec::new(), cursor: Cursor::new(length), watcher: Watcher::new() }
    }

    /// Return a reference to the token output generated from the source string.
    /// 
    /// # Errors
    /// 
    /// This function errors if the provided source string has incorrect tab notation syntax.
    pub fn generate_tokens(&mut self) -> Result<&Vec<Token>, String> {
        if self.tokens.is_empty() {
            while !self.cursor.is_at_end() {
                // reset the start position of the cursor to the current cursor position
                // this allows new tokens to be tokenized from the source string
                self.cursor.start = self.cursor.current;
                self.consume_next();
            }

            // add an EOF token to the token list to signify the end of the file has been reached
            self.tokens.push(
                Token::new(TokenType::EndOfFile, String::new(), Literal::None, self.cursor.line)
            );
        }

        // if there was a syntax error, return an error; otherwise return the token list
        if self.watcher.had_error {
            Err(self.watcher.to_string())
        } else {
            Ok(&self.tokens)
        }
    }

    /// Consumes the next token and generates a new `Token` struct.
    /// 
    /// # Logs Errors
    /// 
    /// This function logs an error if the consumed character is not expected within the tab notation syntax.
    fn consume_next(&mut self) {
        let c: char = self.advance();
        match c {
            '.' => self.add_token(TokenType::Empty, Literal::None),
            ',' => self.add_token(TokenType::Next, Literal::None),
            'A'..='G' => {
                if self.next_matches_modifier() {
                    self.add_token(TokenType::Note, Literal::None);
                } else {
                    self.add_token(TokenType::Note, Literal::None);
                }
            },
            ':' => self.spread(TokenType::SpreadEmpty),
            ';' => self.spread(TokenType::SpreadNext),
            '\n' => { self.cursor.line += 1; },
            '\0'..=' ' => (),
            '[' => self.options(),
            '0'..='9' => self.number(),
            _ => self.watcher.error(self.cursor.line, format!("Unknown character value: {}", c)),
        }
    }

    /// Moves the cursor's current position to the next character and returns it.
    fn advance(&mut self) -> char {
        // get the current cursor position and store it; increment the current position
        let current: usize = self.cursor.current as usize;
        self.cursor.current += 1;

        // get a slice of the source string from the current position and return the first char
        // if a char exists
        if let Some(s) = self.source.get(current..) {
            if let Some(c) = s.chars().next() {
                return c
            }
        }
        
        // if no char exists, return a null char
        '\0'
    }

    /// Checks if the next character is a 'b' or '#' note modifier.
    fn next_matches_modifier(&mut self) -> bool {
        match self.peek() {
            'b' | '#' => {
                self.cursor.current += 1;
                true
            },
            _ => false,
        }
    }

    /// Looks ahead at the next character and returns it.
    fn peek(&self) -> char {
        // get the current cursor position
        let current: usize = self.cursor.current as usize;

        // get a slice of the source string from the current position and return the first char
        // if a char exists
        if let Some(s) = self.source.get(current..) {
            if let Some(c) = s.chars().next() {
                return c
            }
        }
        
        // if no char exists, return a null char
        '\0'
    }

    /// Adds a new token to the token list.
    fn add_token(&mut self, type_of: TokenType, literal: Literal) {
        // get a selection from the cursor's start position and its current position
        let index_range = self.cursor.start as usize..self.cursor.current as usize;
        // add a new token with the current selection range as its value
        if let Some(text) = self.source.get(index_range) {
            self.tokens.push(Token::new(type_of, String::from(text), literal, self.cursor.line));
        }
    }

    /// Adds a spread token to the token list.
    /// 
    /// # Logs Errors
    /// 
    /// This function logs an error if the spread amount cannot be parsed into a `u32` number.
    fn spread(&mut self, spread_type: TokenType) {
        // move cursor's current position over all numbers following the spread token
        while let '0'..='9' = self.peek() {
            self.advance();
        }

        // get a selection from the cursor's start position + 1 and its current position
        let index_range = (self.cursor.start + 1) as usize..self.cursor.current as usize;
        // add a new token with the current selection range as its value
        if let Some(text) = self.source.get(index_range) {
            // attempt to parse the value into a `u32` number to use as the token's literal
            match String::from(text).parse::<u32>() {
                Ok(num_literal) => self.add_token(spread_type, Literal::Number(num_literal)),
                Err(e) => self.watcher.error(
                    self.cursor.line,
                    format!("Could not parse amount \"{}\" for \"{}\": {}", text, spread_type, e)
                ),
            }
        }
    }

    /// Adds an option token to the token list.
    /// 
    /// # Logs Errors
    /// 
    /// This function logs an error if the options sequence is not terminated.
    fn options(&mut self) {
        // move cursor's current position over all characters up until a terminating ']'
        // character is found
        while self.peek() != ']' && !self.cursor.is_at_end() {
            if self.peek() == '\n' { self.cursor.line += 1; }
            self.advance();
        }

        // if the end of the source string is found before the terminating ']' character is found,
        // report a syntax error
        if self.cursor.is_at_end() {
            self.watcher.error(
                self.cursor.line,
                String::from("Unterminated options sequence. Close options sequences with \"]\".")
            );
        } else {
            // consume the ']' character
            self.advance();

            // get a selection from the cursor's start position + 1 and its current position - 1
            let index_range = (self.cursor.start + 1) as usize..(self.cursor.current - 1) as usize;

            // add an options token with the token literal
            self.add_token(TokenType::Options, Literal::Options(
                String::from(match self.source.get(index_range) {
                    Some(t) => t,
                    _ => "",
                })
            ));
        }
    }

    /// Adds a number token to the token list.
    /// 
    /// # Logs Errors
    /// 
    /// This function logs an error if the string slice cannot be parsed into a `u32` number.
    fn number(&mut self) {
        // move cursor's current position over all uninterrupted numbers
        while let '0'..='9' = self.peek() {
            self.advance();
        }

        // get a selection from the cursor's start position and its current position
        let index_range = self.cursor.start as usize..self.cursor.current as usize;
        // add a new token with the current selection range as its value
        if let Some(text) = self.source.get(index_range) {
            // attempt to parse the value into a `u32` number to use as the token's literal
            match String::from(text).parse::<u32>() {
                Ok(num_literal) => self.add_token(TokenType::Number, Literal::Number(num_literal)),
                Err(e) => self.watcher.error(
                    self.cursor.line,
                    format!("String \"{}\" could not be parsed into a number: {}", text, e)
                ),
            }
        }
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn token_output() {
        let mut lex = Lexer::new("E C# Gb\n27 . ,\n:2 ;4 [options]".to_string());
        let expected_tokens = vec![
            Token::new(TokenType::Note, String::from("E"), Literal::None, 1),
            Token::new(TokenType::Note, String::from("C#"), Literal::None, 1),
            Token::new(TokenType::Note, String::from("Gb"), Literal::None, 1),
            Token::new(TokenType::Number, String::from("27"), Literal::Number(27), 2),
            Token::new(TokenType::Empty, String::from("."), Literal::None, 2),
            Token::new(TokenType::Next, String::from(","), Literal::None, 2),
            Token::new(TokenType::SpreadEmpty, String::from(":2"), Literal::Number(2), 3),
            Token::new(TokenType::SpreadNext, String::from(";4"), Literal::Number(4), 3),
            Token::new(TokenType::Options, String::from("[options]"), Literal::Options(String::from("options")), 3),
            Token::new(TokenType::EndOfFile, String::new(), Literal::None, 3),
        ];

        match lex.generate_tokens() {
            Ok(tokens) => {
                for (found, expected) in tokens.iter().zip(expected_tokens.iter()) {
                    assert_eq!(expected, found);
                }
            },
            Err(e) => panic!("Could not generate tokens: {}", e),
        }
    }
}