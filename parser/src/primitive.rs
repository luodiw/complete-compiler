//! Contains functions for parsing individual tokens, such as identifiers and protected keywords.

use common::{ 
    ast::{
        core::ASTNode, data_type::DataType, 
    }, error::ErrorType
};
use crate::core::Parser;
use lexer::token::Token;

impl Parser {
    /// Parses a primitive value token into an AST node representing a literal value.
    ///
    /// # Returns
    ///
    /// Returns an `Option<ASTNode>` containing the literal value, or an error `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the current token is not a `NUMBER` or if there is a failure in token consumption.
    pub fn parse_primitive(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        match self.get_current_token() {
            Some(Token::NUMBER(chars)) => {
                let lit_str: String = chars.iter().collect();
                let node = ASTNode::new(common::ast::node_type::NodeType::Literal(lit_str));
                self.advance();
                Ok(Some(node))
            },
            Some(Token::STRINGLITERAL(chars)) => {
                let lit_str: String = chars.iter().collect();
                let node = ASTNode::new(common::ast::node_type::NodeType::Literal("\"".to_string() + &lit_str + "\""));
                self.advance();
                Ok(Some(node))
            },
            Some(Token::CHAR(c)) => {
                let node = ASTNode::new(common::ast::node_type::NodeType::Literal(format!("'{}'", c)));
                self.advance();
                Ok(Some(node))
            },
            _ => {
                Err(vec![ErrorType::SyntaxError {
                    message: "Expected a literal (number, string, or char)".into(),
                }])
            }
        }
    }

    /// Parses an identifier token into an AST node or an assignment if an equal sign follows the identifier.
    /// This method expects a token of type `IDENTIFIER`.
    ///
    /// # Returns
    ///
    /// Returns an `Option<ASTNode>` containing either the identifier or the assignment node, or an error `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the current token is not an `IDENTIFIER` or if there is a failure in token consumption or assignment parsing.
    /// Parses an identifier token into an AST node or an assignment if an equal sign follows.
    pub fn parse_identifier(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        // Extract the variable name
        let name = self.parse_variable_name()?;

        // Check what follows the identifier
        match self.get_current_token() {
            // If next token is '=', parse assignment
            Some(Token::EQUAL) => {
                // Use the assignment handler with the name we already parsed
                let name_chars: Vec<char> = name.chars().collect();
                self.parse_assignment(name_chars)
            },
            // Otherwise, it's just a bare identifier (or the start of an expression to be handled by a higher-level parser function)
            _ => Ok(Some(ASTNode::new(common::ast::node_type::NodeType::Identifier(name))))
        }
    
    }

    /// Parses a variable name from an identifier token and returns it as a string.
    /// This method expects a token of type `IDENTIFIER`.
    ///
    /// # Returns
    ///
    /// Returns a `String` representing the variable name, or an error `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the current token is not an `IDENTIFIER` or if there is a failure in token consumption.
    pub fn parse_variable_name(&mut self) -> Result<String, Vec<ErrorType>> {
        if let Some(Token::IDENTIFIER(chars)) = self.get_current_token() {
            let name: String = chars.iter().collect();
            self.advance();
            Ok(name)
        } else {
            Err(vec![ErrorType::SyntaxError {
                message: "Expected identifier".into(),
            }])
        }
    }

    /// Parses a protected keyword into the corresponding AST node. Supported keywords include `BREAK`, `CONTINUE`, and `RETURN`.
    /// This method also handles the `EOF` and `SEMICOLON` tokens appropriately.
    ///
    /// # Returns
    ///
    /// Returns an `Option<ASTNode>` containing the parsed keyword node, or an error `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the current token is not a recognized keyword or if there is a failure in token consumption or value parsing.
    pub fn parse_protected_keyword(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        match self.get_current_token() {
            Some(Token::BREAK) => {
                self.consume(Token::BREAK, "Expected 'break'")?;
                self.consume(Token::SEMICOLON, "Expected ';' after 'break'")?;
                Ok(Some(ASTNode::new(common::ast::node_type::NodeType::Break)))
            }
            Some(Token::CONTINUE) => {
                self.consume(Token::CONTINUE, "Expected 'continue'")?;
                self.consume(Token::SEMICOLON, "Expected ';' after 'continue'")?;
                Ok(Some(ASTNode::new(common::ast::node_type::NodeType::Continue)))
            }
            Some(Token::RETURN) => {
                self.consume(Token::RETURN, "Expected 'return'")?;
                
                // Check if there's an expression after 'return'
                let mut return_node = ASTNode::new(common::ast::node_type::NodeType::Return);
                
                match self.get_current_token() {
                    Some(Token::SEMICOLON) => {
                        // Empty return
                        self.consume(Token::SEMICOLON, "Expected ';' after 'return'")?;
                    },
                    _ => {
                        // Return with an expression
                        let expr = match self.get_current_token() {
                            Some(Token::NUMBER(_)) => self.parse_primitive()?,
                            Some(Token::IDENTIFIER(_)) => self.parse_identifier()?,
                            Some(Token::DASH) | Some(Token::EXCLAMATIONPOINT) => self.parse_unary_expression()?,
                            Some(Token::LPAREN) => self.parse_parenthesized_expression()?,
                            _ => {
                                return Err(vec![ErrorType::SyntaxError {
                                    message: "Expected expression after 'return'".into(),
                                }]);
                            }
                        }.ok_or_else(|| vec![ErrorType::SyntaxError {
                            message: "Expected expression after 'return'".into(),
                        }])?;
                        
                        // Wrap the expression in an AssignedValue node as expected by the tests
                        let mut assigned_value = ASTNode::new(common::ast::node_type::NodeType::AssignedValue);
                        assigned_value.add_child(expr);
                        return_node.add_child(assigned_value);
                        
                        // Consume the semicolon after the expression
                        self.consume(Token::SEMICOLON, "Expected ';' after return expression")?;
                    }
                }
                
                Ok(Some(return_node))
            }
            Some(Token::SEMICOLON) | Some(Token::EOF) => {
                // Empty statement or end
                Ok(None)
            }
            _ => Err(vec![ErrorType::SyntaxError {
                message: "Expected break, continue, or return".into(),
            }])
        }
    }

    /// Consumes a type token and returns the corresponding `DataType` enum value. Supported types include
    /// `TINTEGER`, `TBOOLEAN`, `TDOUBLE`, `TFLOAT`, `TCHAR`, `TVOID`, `TSIGN`, `TUSIGN`, `TSIGNINT`, and `TLONG`.
    ///
    /// # Returns
    ///
    /// Returns a `DataType` representing the type of the token, or an error `ErrorType` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if the current token is not a recognized type token or if there is a failure in token consumption.
    pub fn parse_type(&mut self) -> Result<DataType, ErrorType> {
        // Peek at the current token to decide which DataType it represents
        let dt = match self.get_current_token() {
            Some(Token::TINTEGER)  => DataType::Integer,
            Some(Token::TBOOLEAN)  => DataType::Boolean,
            Some(Token::TDOUBLE)   => DataType::Double,
            Some(Token::TFLOAT)    => DataType::Float,
            Some(Token::TCHAR)     => DataType::Char,
            Some(Token::TVOID)     => DataType::Void,
            Some(Token::TSIGNINT)  => DataType::Sign,
            Some(Token::TUSIGN)    => DataType::Unsign,
            Some(Token::TLONG)     => DataType::Long,
            _ => {
                return Err(ErrorType::SyntaxError {
                    message: "Expected a type keyword (`int`, `boolean`, etc.)".into(),
                });
            }
        };
    
        // Consume the type token now that we've recorded it
        self.advance();
        Ok(dt)
    }
}