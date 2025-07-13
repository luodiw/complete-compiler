//! This file hosts all of the functions necessary for generating LLVM IR
//! for statement nodes.

use common::{
    ast::{
        core::ASTNode, data_type::DataType, node_type::NodeType
    }, error::ErrorType,
};

use crate::core::IRGenerator;
use safe_llvm::ir::core::Tag;

impl IRGenerator {
    /// Generates LLVM IR for a statement.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a statement.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing the Tag of this statement
    /// if generation went smoothly or an Error if there was a problem generating the statement.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a statement from */
    /// //let result = self.generate_statement_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain the Tag that houses the 
    /// statement's ValueTag. */
    /// ```
    pub fn generate_statement_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let child_node = &node.get_children()[0];
        self.ir_router(child_node)
    }

    /// Generates LLVM IR for an assignment.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for an assignment.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the assignment.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate an assignment from */
    /// //let result = self.generate_assignment_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_assignment_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let children = node.get_children();
        // Accept either an Identifier or a Variable node as the assignee
        let assignee_name = match children[0].get_node_type() {
            NodeType::Identifier(name) => name,
            NodeType::Variable => {
                let id_child = &children[0].get_children()[0];
                match id_child.get_node_type() {
                    NodeType::Identifier(name) => name,
                    _ => return Err(ErrorType::DevError { message: "Expected identifier in variable node".to_string() })
                }
            },
            _ => return Err(ErrorType::DevError { message: "Expected identifier or variable node".to_string() })
        };

        // Process value first
        let llvm_value = self.ir_router(&children[1])?.expect("Missing value in assignment");
        let llvm_value = match llvm_value {
            Tag::Value(value) => value,
            _ => return Err(ErrorType::DevError { message: "Expected value tag".to_string() })
        };
        
        // Get allocation with proper mutex handling
        let llvm_alloca = self.search_store_table(assignee_name.clone());
        
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.lock().expect("Failed to lock mutex in assignment!");
        
        resource_pools.reassign_var(self.get_builder(), llvm_alloca, llvm_value)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to reassign variable".to_string() })?;

        Ok(None)
    }

    /// Generates LLVM IR for a variable initialization.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a variable initialization.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the initialization.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate an initialization from */
    /// //let result = self.generate_initialization_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_initialization_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let children = node.get_children();
        if children.len() < 2 || children.len() > 3 {
            return Err(ErrorType::DevError { 
                message: "Invalid variable initialization node: expected 2 or 3 children".to_string() 
            });
        }

        // Handle both direct identifier and variable node cases
        let var_name = match children[0].get_node_type() {
            NodeType::Identifier(name) => name.clone(),
            NodeType::Variable => {
                let id_child = &children[0].get_children()[0];
                match id_child.get_node_type() {
                    NodeType::Identifier(name) => name.clone(),
                    _ => return Err(ErrorType::DevError { 
                        message: "Expected identifier in variable node".to_string() 
                    })
                }
            },
            _ => return Err(ErrorType::DevError { 
                message: "Expected identifier or variable node".to_string() 
            })
        };

        // Process type node or infer type from initial value
        let (type_tag, init_value_node_opt) = if children.len() == 3 {
            // 3 children: [var, type, value]
            let type_node = &children[1];
            let init_value_node = &children[2];
            let type_tag = match type_node.get_node_type() {
                NodeType::Type(data_type) => {
                    let resource_pools = self.get_resource_pools();
                    let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in type processing!");
                    match data_type {
                        DataType::Integer => resource_pools.int_type(self.get_context(), 64)
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer type".to_string() })?,
                        DataType::Float => resource_pools.float_type(self.get_context())
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create float type".to_string() })?,
                        DataType::Boolean => resource_pools.boolean_type(self.get_context())
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create boolean type".to_string() })?,
                        DataType::Void => resource_pools.void_type(self.get_context())
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create void type".to_string() })?,
                        _ => return Err(ErrorType::DevError { message: format!("Unsupported data type: {:?}", data_type) })
                    }
                },
                _ => {
                    let resource_pools = self.get_resource_pools();
                    let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in type processing!");
                    resource_pools.int_type(self.get_context(), 64)
                        .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer type".to_string() })?
                }
            };
            (type_tag, Some(init_value_node))
        } else if children.len() == 2 {
            // 2 children: [var, value] or [var, type]
            match children[1].get_node_type() {
                NodeType::Type(data_type) => {
                    let resource_pools = self.get_resource_pools();
                    let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in type processing!");
                    let type_tag = match data_type {
                        DataType::Integer => resource_pools.int_type(self.get_context(), 64)
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer type".to_string() })?,
                        DataType::Float => resource_pools.float_type(self.get_context())
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create float type".to_string() })?,
                        DataType::Boolean => resource_pools.boolean_type(self.get_context())
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create boolean type".to_string() })?,
                        DataType::Void => resource_pools.void_type(self.get_context())
                            .ok_or_else(|| ErrorType::DevError { message: "Failed to create void type".to_string() })?,
                        _ => return Err(ErrorType::DevError { message: format!("Unsupported data type: {:?}", data_type) })
                    };
                    (type_tag, None)
                },
                _ => {
                    // If not a type, treat as initial value and default to i64
                    let resource_pools = self.get_resource_pools();
                    let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in type processing!");
                    let type_tag = resource_pools.int_type(self.get_context(), 64)
                        .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer type".to_string() })?;
                    (type_tag, Some(&children[1]))
                }
            }
        } else {
            return Err(ErrorType::DevError { message: "Invalid variable initialization node: expected 2 or 3 children".to_string() });
        };

        // Helper: recursively check for AssignedValue -> Variable
        fn get_variable_node_from_assigned_value(node: &ASTNode) -> Option<ASTNode> {
            match node.get_node_type() {
                NodeType::AssignedValue => {
                    let children = node.get_children();
                    if children.len() == 1 {
                        let child = &children[0];
                        match child.get_node_type() {
                            NodeType::Variable => Some(child.clone()),
                            NodeType::AssignedValue => get_variable_node_from_assigned_value(child),
                            _ => None
                        }
                    } else {
                        None
                    }
                },
                NodeType::Variable => Some(node.clone()),
                _ => None
            }
        }

        // Special case: if the initial value is (or wraps) a variable, emit the load before the alloca for the new variable
        if let Some(init_value_node) = init_value_node_opt {
            if let Some(var_node) = get_variable_node_from_assigned_value(init_value_node) {
                // 1. Load from the source variable FIRST
                let src_var_name = match var_node.get_children()[0].get_node_type() {
                    NodeType::Identifier(ref n) => n.clone(),
                    _ => return Err(ErrorType::DevError { message: "Expected identifier in variable node".to_string() })
                };
                let src_alloca = self.search_store_table(src_var_name);
                let resource_pools = self.get_resource_pools();
                let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in variable recall!");
                let type_tag = resource_pools.int_type(self.get_context(), 64)
                    .ok_or_else(|| ErrorType::DevError { message: "Failed to create i64 type".to_string() })?;
                let loaded = resource_pools.get_var(self.get_builder(), type_tag, src_alloca, "vrecallID1")
                    .ok_or_else(|| ErrorType::DevError { message: "Failed to load variable".to_string() })?;
                drop(resource_pools);

                // 2. THEN alloca for the new variable
                let resource_pools = self.get_resource_pools();
                let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in initialization!");
                let alloca = resource_pools.init_var(self.get_builder(), &var_name, type_tag, None)
                    .ok_or_else(|| ErrorType::DevError { message: "Failed to initialize variable".to_string() })?;
                drop(resource_pools);
                self.add_tag_to_store_table(var_name.clone(), alloca);

                // 3. FINALLY store the loaded value
                let resource_pools = self.get_resource_pools();
                let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in initialization store!");
                resource_pools.reassign_var(self.get_builder(), alloca, loaded)
                    .ok_or_else(|| ErrorType::DevError { message: "Failed to store initial value".to_string() })?;
                return Ok(None);
            }
        }

        // Default case: alloca, then store (if any)
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in initialization!");
        let alloca = resource_pools.init_var(self.get_builder(), &var_name, type_tag, None)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to initialize variable".to_string() })?;
        drop(resource_pools);
        self.add_tag_to_store_table(var_name.clone(), alloca);

        // If there is an initial value, emit a store
        if let Some(init_value_node) = init_value_node_opt {
            let llvm_value = self.ir_router(init_value_node)?
                .ok_or_else(|| ErrorType::DevError { message: "Failed to generate initial value".to_string() })?;
            let store_value = match llvm_value {
                Tag::Value(value_tag) => value_tag,
                _ => return Err(ErrorType::DevError { message: "Expected value tag from initial value node".to_string() })
            };
            let resource_pools = self.get_resource_pools();
            let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in initialization store!");
            resource_pools.reassign_var(self.get_builder(), alloca, store_value)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to store initial value".to_string() })?;
        }

        Ok(None)
    }

    /// Generates LLVM IR for a break statement.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a break statement.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the break.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a break from */
    /// //let result = self.generate_break_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_break_ir(&mut self, _node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let targets = self.get_break_continue_target()
            .ok_or_else(|| ErrorType::DevError { message: "No break/continue targets available".to_string() })?;
        
        let break_target = targets.get(0)
            .ok_or_else(|| ErrorType::DevError { message: "No break target available".to_string() })?
            .clone();
        
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in break!");
        
        resource_pools.create_br(self.get_builder(), break_target)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create break branch".to_string() })?;
        
        Ok(None)
    }

    /// Generates LLVM IR for a continue statement.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a continue statement.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the continue.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a continue from */
    /// //let result = self.generate_continue_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_continue_ir(&mut self, _node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let targets = self.get_break_continue_target()
            .ok_or_else(|| ErrorType::DevError { message: "No break/continue targets available".to_string() })?;
        
        let continue_target = targets.get(1)
            .ok_or_else(|| ErrorType::DevError { message: "No continue target available".to_string() })?
            .clone();
        
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in continue!");
        
        resource_pools.create_br(self.get_builder(), continue_target)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create continue branch".to_string() })?;
        
        Ok(None)
    }

    /// Generates LLVM IR for a return statement.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a return statement.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the return.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    pub fn generate_return_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in return!");

        let children = node.get_children();
        if !children.is_empty() {
            // Release lock before calling self methods
            drop(resource_pools);
            
            let value_ptr = self.ir_router(&children[0])?;
            let value_ptr = value_ptr.expect("Missing return value");
            let llvm_value = match value_ptr {
                Tag::Value(value) => value,
                _ => return Err(ErrorType::DevError { message: "Expected value tag".to_string() })
            };
            
            // Re-acquire lock
            let resource_pools = self.get_resource_pools();
            let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in return!");
            
            resource_pools.nonvoid_return(self.get_builder(), llvm_value)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create return instruction".to_string() })?;
        } else {
            resource_pools.void_return(self.get_builder())
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create void return instruction".to_string() })?;
        }

        Ok(None)
    }

    /// Generates LLVM IR for a variable recall.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a variable recall.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing the Tag of this variable recall
    /// if generation went smoothly or an Error if there was a problem generating the var recall.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    pub fn generate_variable_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        // Accept only a Variable node
        let name = match node.get_node_type() {
            NodeType::Variable => {
                let id_child = &node.get_children()[0];
                match id_child.get_node_type() {
                    NodeType::Identifier(name) => name,
                    _ => return Err(ErrorType::DevError { message: "Expected identifier in variable node".to_string() })
                }
            },
            _ => return Err(ErrorType::DevError { message: "Expected variable node".to_string() })
        };

        let llvm_alloca = self.search_store_table(name.clone());
        
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in variable recall!");

        // For now, assume i64 type for variables
        // TODO: Get actual type from symbol table or node metadata
        let type_tag = resource_pools.int_type(self.get_context(), 64)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create i64 type".to_string() })?;

        // Load the value from the variable
        let load = resource_pools.get_var(self.get_builder(), type_tag, llvm_alloca, "vrecallID1")
            .ok_or_else(|| ErrorType::DevError { message: "Failed to load variable".to_string() })?;

        Ok(Some(Tag::Value(load)))
    }
}