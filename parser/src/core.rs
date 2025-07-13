//! The driver for the parsing process, uses the method of recursive descent to systematically iterate through 
//! tokens and routes to appropriate helper methods in the parser to construct an abstract syntax tree.
                                 
use common::{ 
    ast::core::{ASTNode, AST}, 
    error::ErrorType
};
use lexer::token::Token;

/// The `Parser` struct models the process of parsing.
/// 
/// At initialization, it takes an input as a vector of tokens.
///
/// # Fields
/// * `input` - A vector of tokens from the output of the lexer representing the source code to be parsed.
/// * `current` - The current token being considered by the parser.
pub struct Parser {
    input: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Creates a new `Parser` instance with the given input tokens.
    ///
    /// This initializer sets up a `Parser` by accepting a vector of tokens and initializing the
    /// current token index to 0.
    ///
    /// # Parameters
    ///
    /// * `input`: A vector of `Token` representing the sequence of tokens to be parsed.
    ///
    /// # Returns
    ///
    /// Returns a new `Parser` instance ready to parse the provided tokens.
    fn new(input: Vec<Token>) -> Self {
        Self {
            input,
            current: 0,
        }
    }


     /// Advances the parser position by one token.
     pub(crate) fn advance(&mut self) {
        if self.current < self.input.len() {
            self.current += 1;
        }
    }

    pub(crate) fn get_current_token(&mut self) -> Option<&Token> {
        if self.current < self.input.len() {
            Some(&self.input[self.current])
        } else {
            None
        }
    }

    pub(crate) fn peek_next_token(&mut self) -> Option<&Token> {
        if self.current + 1 < self.input.len() {
            Some(&self.input[self.current + 1])
        } else {
            None
        }
    }

    // Consume the current token if it equals `expected`, advancing past it.
    /// Otherwise return a single‐element Vec<ErrorType> with your `message`.
    pub(crate) fn consume(&mut self, expected: Token, message: &str) -> Result<(), Vec<ErrorType>> {
        match self.get_current_token() {
            Some(tok) if *tok == expected => {
                self.advance();
                Ok(())
            }
            _ => Err(vec![ErrorType::SyntaxError {
                message: message.to_string(),
            }]),
        }
    }

    /// Parses an input of tokens into an AST using recursive descent parsing.
    /// Iterates through tokens and routes to appropriate helper methods to construct an AST.
    ///
    /// # Parameters
    ///
    /// * `input`: A vector of `Token` representing the input to be parsed.
    ///
    /// # Returns
    ///
    /// Returns a `Result<AST, Vec<ErrorType>>` containing the constructed AST if successful, 
    /// or a vector of `ErrorType` if there are parsing errors.
    ///
    /// # Errors
    ///
    /// * Returns a vector of errors if there are issues during parsing, such as unexpected tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// use lexer::token::Token;
    /// use parser::core::Parser;
    /// let tokens: Vec<Token> = vec![/* tokens */];
    /// let ast = Parser::parse(tokens);
    /// ```
    pub fn parse(input: Vec<Token>) -> Result<AST, Vec<ErrorType>> {
        let mut parser = Parser::new(input);
        let mut children = vec![];
        
        while let Some(token) = parser.get_current_token() {
            match token {
                Token::EOF => break,
                _ => {
                    match parser.parse_router()? {
                        Some(node) => children.push(node),
                        None => parser.advance(),
                    }
                }
            }
        }

        let mut root = ASTNode::new(common::ast::node_type::NodeType::TopLevelExpression);
        root.set_children(children);
        Ok(AST::new(root))
    }

   

    /// Entry point to the main parsing logic. Routes the current token to the appropriate parsing method based on token type.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<ASTNode>, Vec<ErrorType>>` containing the parsed AST node or errors encountered during parsing.
    ///
    /// # Errors
    ///
    /// * Returns a vector of errors if there are issues during parsing, such as unexpected tokens or parsing failures.
    pub fn parse_router(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        if self.current >= self.input.len() {
            return Ok(None);
        }
    
        match self.get_current_token() {
            // End‐of‐input marker
            Some(Token::EOF) => Ok(None),
    
            // Skip stray semicolons at top level
            Some(Token::SEMICOLON) => {
                self.advance();
                Ok(None)
            },
    
            // Block delimiters
            Some(Token::LBRACKET) => self.parse_block(),
            Some(Token::RBRACKET) => {
                self.advance(); 
                Ok(None)
            },
    
            // Literals
            Some(Token::NUMBER(_)) => self.parse_primitive(),
            Some(Token::STRINGLITERAL(_)) => self.parse_primitive(), 
            Some(Token::CHAR(_)) => self.parse_primitive(), 
    
            // Identifiers, assignments, or start of binary/unary expressions
            Some(Token::IDENTIFIER(_)) => {
                // Always try to parse as a binary expression first
                self.parse_binary_expression()
            },
    
            // Expressions starting with unary operators should still be parsed as full expressions to
            // correctly capture cases like `-5 - 3`. The precedence-climbing logic internally calls
            // `parse_unary_expression` for the left-hand side.
            Some(Token::DASH) | Some(Token::EXCLAMATIONPOINT) => self.parse_binary_expression(),
            
            // Control flow statements
            Some(Token::IF) => self.parse_if_statement(),
            Some(Token::FOR) => self.parse_for_loop(),
            Some(Token::WHILE) => self.parse_while_loop(),
            Some(Token::DO) => self.parse_do_while_loop(),
            Some(Token::SWITCH) => self.parse_switch_statement(),
            Some(Token::CASE) => self.parse_case(), 
            Some(Token::DEFAULT) => self.parse_default(), 
            
            // Declarations
            Some(Token::STRUCT) => self.parse_struct_declaration(),
            Some(Token::ENUM) => self.parse_enum_declaration(),
            
            // break / continue / return
            Some(Token::BREAK) | Some(Token::CONTINUE) | Some(Token::RETURN) => {
                self.parse_protected_keyword()
            },
    
            // Leading‐type → var‐ or func‐decl
            Some(Token::TINTEGER)
            | Some(Token::TBOOLEAN)
            | Some(Token::TDOUBLE)
            | Some(Token::TFLOAT)
            | Some(Token::TCHAR)
            | Some(Token::TVOID)
            | Some(Token::TSIGNINT)
            | Some(Token::TUSIGN)
            | Some(Token::TLONG) => self.parse_initialization(),
            
            // Binary operators
            Some(Token::PLUS) | Some(Token::ASTERISK) | Some(Token::FSLASH) |
            Some(Token::LESSTHAN) | Some(Token::GREATERTHAN) |
            Some(Token::EQUALEQUAL) | Some(Token::NOTEQUAL) => self.parse_binary_expression(),
            
            // Assignment operators
            Some(Token::PLUSPLUS) | Some(Token::MINUSMINUS) => {
                self.parse_unary_expression()
            },
    
            // Logical operators
            Some(Token::ANDAND) | Some(Token::BARBAR) => {
                self.parse_binary_expression()
            },
    
            // Parenthesized expression at top level
            Some(Token::LPAREN) => {
                // Delegate to binary expression parsing which internally handles parenthesized sub‐expressions.
                // This allows expressions such as `(a * b) + c` to be parsed in a single expression tree
                // instead of treating the parentheses as a control‐flow condition.
                self.parse_binary_expression()
            },
    
            // Errors
            Some(tok) => Err(vec![ErrorType::SyntaxError {
                message: format!("Unexpected token in top‐level: {:?}", tok),
            }]),
    
            // Empty token
            None => Ok(None),
        }
    }

    
}