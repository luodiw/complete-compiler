//! Contains functions for parsing expressions, such as unary expressions, binary expressions, variable assignments and parenthesized expressions.

use common::{ 
    error::ErrorType,
    ast::{core::ASTNode, node_type::NodeType},
};
use crate::core::Parser;
use lexer::token::Token;

impl Parser {
    /// Parses a unary expression. 
    /// Specifically handles DASH and EXCLAMATIONPOINT tokens, as returns corresponding AST with a top-level 
    /// 'NodeType::UnaryExpression' ASTNode.
    ///
    /// # Returns
    ///
    /// Returns an `Option<ASTNode>` representing the parsed unary expression, or an error
    /// `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if parsing of the unary expression fails.
    pub fn parse_unary_expression(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        // Check if the current token is a unary operator (- or !)
        let operator = match self.get_current_token() {
            Some(Token::DASH) => {
                self.advance();
                "-".to_string()
            },
            Some(Token::EXCLAMATIONPOINT) => {
                self.advance();
                "!".to_string()
            },
            _ => {
                return Err(vec![ErrorType::SyntaxError {
                    message: "Expected unary operator (- or !)".into(),
                }]);
            }
        };

        // Parse the operand (can be a primitive, another expression, or parenthesized expression)
        let operand = match self.get_current_token() {
            Some(Token::NUMBER(_)) => self.parse_primitive()?,
            Some(Token::IDENTIFIER(_)) => self.parse_identifier()?,
            Some(Token::LPAREN) => self.parse_parenthesized_expression()?,
            _ => {
                return Err(vec![ErrorType::SyntaxError {
                    message: "Expected expression after unary operator".into(),
                }]);
            }
        }.ok_or_else(|| vec![ErrorType::SyntaxError {
            message: "Expected expression after unary operator".into(),
        }])?;

        // Create unary expression node
        let mut unary_expr = ASTNode::new(NodeType::UnaryExpression);
        unary_expr.add_child(ASTNode::new(NodeType::Operator(operator)));
        unary_expr.add_child(operand);
        
        // Simply return the unary expression. Any following binary operators will be handled by
        // `parse_expression_with_precedence`, which ensures correct operator precedence.
        Ok(Some(unary_expr))
    }

    /// Parses a variable reassignment. Handles assignment to literals, expressions, and other identifiers.
    /// Creates a top level 'NodeType::Assignment' ASTNode, with children representing the identifier and
    /// its new AssignedValue. Is called by 'Parser::parse_identifier', which fullfills the `name_chars` parameter.
    ///
    /// # Parameters
    ///
    /// * `name_chars`: A vector of characters representing the name of the variable to be reassigned.
    ///
    /// # Returns
    ///
    /// Returns an `Option<ASTNode>` representing the parsed assignment, or an error
    /// `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if parsing of the assignment fails.
    pub fn parse_assignment(&mut self, name_chars: Vec<char>) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        // Create an identifier from the name
        let name: String = name_chars.iter().collect();
        
        // Consume the equal sign
        self.consume(Token::EQUAL, "Expected '=' for assignment")?;
        
        // Parse the expression on the right side of the equals sign
        // This can be a simple value or a complex expression
        let mut assigned_value = match self.get_current_token() {
            Some(Token::NUMBER(_)) => self.parse_primitive()?,
            Some(Token::IDENTIFIER(_)) => self.parse_identifier()?,
            Some(Token::DASH) | Some(Token::EXCLAMATIONPOINT) => self.parse_unary_expression()?,
            Some(Token::LPAREN) => self.parse_parenthesized_expression()?,
            _ => {
                return Err(vec![ErrorType::SyntaxError {
                    message: "Expected expression after '='".into(),
                }]);
            }
        }.ok_or_else(|| vec![ErrorType::SyntaxError {
            message: "Expected expression after '='".into(),
        }])?;

        // Check if there's a binary operator after the initial value (e.g., "x = 10 * 4")
        // Handle complex expressions like "x = a * (b + c)"
        loop {
            match self.get_current_token() {
                Some(Token::PLUS) | Some(Token::DASH) | Some(Token::ASTERISK) | Some(Token::FSLASH) => {
                    // Parse the operator
                    let operator = match self.get_current_token() {
                        Some(Token::PLUS) => {
                            self.advance();
                            "+".to_string()
                        },
                        Some(Token::DASH) => {
                            self.advance();
                            "-".to_string()
                        },
                        Some(Token::ASTERISK) => {
                            self.advance();
                            "*".to_string()
                        },
                        Some(Token::FSLASH) => {
                            self.advance();
                            "/".to_string()
                        },
                        _ => unreachable!(),
                    };
                    
                    // Parse the right side of the binary expression
                    let right = match self.get_current_token() {
                        Some(Token::NUMBER(_)) => self.parse_primitive()?,
                        Some(Token::IDENTIFIER(_)) => self.parse_identifier()?,
                        Some(Token::DASH) | Some(Token::EXCLAMATIONPOINT) => self.parse_unary_expression()?,
                        Some(Token::LPAREN) => self.parse_parenthesized_expression()?,
                        _ => {
                            return Err(vec![ErrorType::SyntaxError {
                                message: "Expected expression after operator".into(),
                            }]);
                        }
                    }.ok_or_else(|| vec![ErrorType::SyntaxError {
                        message: "Expected expression after operator".into(),
                    }])?;
                    
                    // Create a binary expression with proper precedence
                    // If we have a * (b + c), ensure it has the right structure
                    let mut binary_expr = ASTNode::new(NodeType::BinaryExpression);
                    binary_expr.add_child(assigned_value);
                    binary_expr.add_child(ASTNode::new(NodeType::Operator(operator)));
                    binary_expr.add_child(right);
                    assigned_value = binary_expr;
                },
                _ => break, // Exit the loop if no more operators
            }
        }

        // Create the assignment node
        let mut assignment_node = ASTNode::new(NodeType::Assignment);
        assignment_node.add_child(ASTNode::new(NodeType::Identifier(name)));
        assignment_node.add_child(assigned_value);
        
        // Consume semicolon if present
        if let Some(Token::SEMICOLON) = self.get_current_token() {
            self.consume(Token::SEMICOLON, "Expected ';' after assignment")?;
        }
        
        Ok(Some(assignment_node))
    }

    /// Entry point for the parsing of a binary expression.
    ///
    /// # Returns
    /// 
    /// * `Ok(Some(ASTNode))` - if the binary expression was successfully parsed.
    /// * `Ok(None)` - if there was no binary expression to parse.
    /// * `Err(Vec<ErrorType>)` - if there were errors encountered during parsing.
    ///
    /// # Errors
    ///
    /// * Returns an error if parsing of the assignment fails.
    pub fn parse_binary_expression(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        self.parse_expression_with_precedence(0)
    }

    /// Helper function to parse expressions with operator precedence.
    /// Uses the precedence climbing method to correctly handle operator precedence.
    fn parse_expression_with_precedence(&mut self, min_precedence: i32) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        // Parse the left-hand side
        let mut left = match self.get_current_token() {
            Some(Token::NUMBER(_)) => self.parse_primitive()?,
            Some(Token::IDENTIFIER(_)) => self.parse_identifier()?,
            Some(Token::DASH) | Some(Token::EXCLAMATIONPOINT) => self.parse_unary_expression()?,
            Some(Token::LPAREN) => self.parse_parenthesized_expression()?,
            _ => {
                return Err(vec![ErrorType::SyntaxError {
                    message: "Expected expression".into(),
                }]);
            }
        }.ok_or_else(|| vec![ErrorType::SyntaxError {
            message: "Expected left-hand side expression".into(),
        }])?;

        // Define operator precedence
        let get_precedence = |op: &str| -> i32 {
            match op {
                "*" | "/" | "%" => 3,
                "+" | "-" => 2,
                "<" | ">" | "<=" | ">=" => 1,
                "==" | "!=" => 0,
                _ => -1,
            }
        };

        // While we have operators with higher precedence than min_precedence
        while let Some(token) = self.get_current_token() {
            // Determine the operator string **without** advancing so we can check precedence first
            let operator = match token {
                Token::PLUS          => "+".to_string(),
                Token::DASH          => "-".to_string(),
                Token::ASTERISK      => "*".to_string(),
                Token::FSLASH        => "/".to_string(),
                Token::PERCENT       => "%".to_string(),
                Token::LESSTHAN      => "<".to_string(),
                Token::GREATERTHAN   => ">".to_string(),
                Token::EQUALEQUAL    => "==".to_string(),
                Token::NOTEQUAL      => "!=".to_string(),
                _ => break,
            };

            // Check precedence before consuming the operator so we don't accidentally skip it
            let op_precedence = get_precedence(&operator);
            if op_precedence < min_precedence {
                break;
            }

            // Now consume the operator **after** validating precedence
            self.advance();

            // Parse the right-hand side with higher precedence (op_precedence + 1)
            let right = self.parse_expression_with_precedence(op_precedence + 1)?;
            let right = right.ok_or_else(|| vec![ErrorType::SyntaxError {
                message: "Expected right-hand side expression".into(),
            }])?;

            // Build the binary expression node
            let mut binary_expr = ASTNode::new(NodeType::BinaryExpression);
            binary_expr.add_child(left);
            binary_expr.add_child(ASTNode::new(NodeType::Operator(operator)));
            binary_expr.add_child(right);

            left = binary_expr;
        }
        
        Ok(Some(left))
    }

    /// Parses a parenthesized expression, which is an expression enclosed in parentheses.
    /// This is used for grouping expressions to override default operator precedence.
    /// Handles complex expressions like (3 + 4) * 2, (1 + 2) * (3 - 4), and ((7 + 8) * 2) / 3.
    ///
    /// # Returns
    ///
    /// Returns an `Option<ASTNode>` representing the parsed expression, or an error
    /// `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if parsing of the expression fails.
    pub fn parse_parenthesized_expression(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        // Consume the opening parenthesis
        self.consume(Token::LPAREN, "Expected '(' for parenthesized expression")?;

        // Parse the full expression inside the parentheses using normal binary-expression parsing
        let expr = self.parse_binary_expression()?;
        let expr = expr.ok_or_else(|| vec![ErrorType::SyntaxError {
            message: "Expected expression within parentheses".into(),
        }])?;

        // Consume the closing parenthesis
        self.consume(Token::RPAREN, "Expected ')' to close parenthesized expression")?;

        // Return the inner expression; any following operators will be handled by the
        // surrounding `parse_expression_with_precedence` call.
        Ok(Some(expr))
    }
    
    /// Parses a condition expression, which is often part of control flow statements.
    ///
    /// # Returns
    ///
    /// Returns an `Option<ASTNode>` representing the parsed condition, or an error
    /// `Vec<ErrorType>` if parsing fails.
    ///
    /// # Errors
    ///
    /// * Returns an error if parsing of the condition fails.
    pub fn parse_condition(&mut self) -> Result<Option<ASTNode>, Vec<ErrorType>> {
        self.consume(Token::LPAREN, "Expected '(' after control flow keyword")?;
        
        // Parse the condition expression
        let condition_expr = match self.get_current_token() {
            Some(Token::NUMBER(_)) => self.parse_primitive()?,
            Some(Token::IDENTIFIER(_)) => self.parse_identifier()?,
            Some(Token::DASH) | Some(Token::EXCLAMATIONPOINT) => self.parse_unary_expression()?,
            _ => {
                self.parse_binary_expression()?
            }
        }.ok_or_else(|| vec![ErrorType::SyntaxError {
            message: "Expected condition expression".into(),
        }])?;
        
        self.consume(Token::RPAREN, "Expected ')' after condition")?;
        
        // Create the condition node
        let mut condition_node = ASTNode::new(NodeType::Condition);
        condition_node.add_child(condition_expr);
        
        Ok(Some(condition_node))
    }
}