//! This file hosts all of the functions necessary for generating LLVM IR
//! for "block" nodes, nodes that generate and manipulate basic blocks.

use common::{
    ast::{
        core::ASTNode, data_type::DataType, node_type::NodeType
    }, constants::{DEFAULT_DO_BODY_LABEL, DEFAULT_DO_CONDITION_LABEL, DEFAULT_DO_WHILE_END_LABEL, DEFAULT_ELSE_LABEL, DEFAULT_ENTRY_LABEL, DEFAULT_FOR_BODY_LABEL, DEFAULT_FOR_COND_LABEL, DEFAULT_FOR_END_LABEL, DEFAULT_FOR_INCREMENT_LABEL, DEFAULT_MERGE_LABEL, DEFAULT_THEN_LABEL, DEFAULT_WHILE_BODY_LABEL, DEFAULT_WHILE_COND_LABEL, DEFAULT_WHILE_END_LABEL}, error::ErrorType
};

use safe_llvm::ir::core::{Tag, ValueTag};
use safe_llvm::common::pointer::{LLVMRef, LLVMRefType};
use crate::core::IRGenerator;

impl IRGenerator {
    /// Generates LLVM IR for a function declaration.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a function declaration.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing the Tag of this function
    /// if generation went smoothly or an Error if there was a problem generating the function.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a function declaration from */
    /// //let result = self.generate_fn_declaration_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain the Tag that houses the 
    /// function's ValueTag. */
    /// ```
    pub fn generate_fn_declaration_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let children = node.get_children();
        if children.len() != 3 {
            return Err(ErrorType::DevError { message: "Invalid function declaration node".to_string() });
        }

        let name_node = &children[0];
        let type_node = &children[1];
        let block_node = &children[2];

        let name = match name_node.get_node_type() {
            NodeType::Identifier(name) => name,
            _ => return Err(ErrorType::DevError { message: "Expected identifier node".to_string() })
        };

        let type_ptr = self.ir_router(type_node)?;
        let type_ptr = type_ptr.expect("Missing type");

        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in function declaration!");

        let return_type = match type_ptr {
            Tag::Type(ty) => ty,
            _ => return Err(ErrorType::DevError { message: "Expected type tag".to_string() })
        };

        // Create a function type with the return type and no parameters (for now)
        let fn_type = resource_pools.create_function(Some(return_type), &[], false, self.get_context())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create function type".to_string() })?;

        let module_tag = self.get_module();

        let func_tag = resource_pools.add_function_to_module(module_tag, &name, fn_type)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to add function to module".to_string() })?;

        // Set this as the current function
        self.set_function(func_tag);

        let label = format!("entryID{}", self.get_next_label_id());
        let entry_block = resource_pools.create_basic_block(self.get_context(), func_tag, &label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create entry block".to_string() })?;

        resource_pools.position_builder_at_end(self.get_builder(), entry_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;

        // Release lock before processing block
        drop(resource_pools);

        let _ = self.ir_router(block_node)?;

        Ok(None)
    }
    
    /// Generates LLVM IR for a block expression.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a block expression.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the block expression.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a block expression from */
    /// //let result = self.generate_block_exp(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_block_exp(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        for child in node.get_children() {
            let _ = self.ir_router(&child)?;
        }
        Ok(None)
    }

    /// Generates LLVM IR for a do while loop.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a do while loop.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the do while.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a do while loop from */
    /// //let result = self.generate_do_while_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_do_while_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let children = node.get_children();
        
        // Handle do-while loops with different numbers of children
        let (body_node_opt, cond_node_opt) = match children.len() {
            0 => (None, None), // Empty loop (just for completeness)
            1 => (Some(&children[0]), None), // Just body
            2 => (Some(&children[0]), Some(&children[1])), // Body and condition
            _ => return Err(ErrorType::DevError { 
                message: format!("Invalid do-while node: unexpected number of children {}", children.len()) 
            })
        };

        let function = self.get_function().unwrap();
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in do-while!");

        let next_id = self.get_next_label_id();
        let body_label = format!("do_bodyID{}", next_id);
        let cond_label = format!("do_condID{}", next_id);
        let end_label = format!("do_endID{}", next_id);

        // Create blocks in the correct order
        let current_insert = self.get_current_insert_block().unwrap_or_else(|| {
            resource_pools.get_current_block(self.get_builder()).expect("No current block!")
        });

        let body_block = resource_pools.create_basic_block_after(self.get_context(), function, current_insert, &body_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create body block".to_string() })?;
        let cond_block = resource_pools.create_basic_block_after(self.get_context(), function, body_block, &cond_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create condition block".to_string() })?;
        let end_block = resource_pools.create_basic_block_after(self.get_context(), function, cond_block, &end_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create end block".to_string() })?;

        // Branch to body block
        resource_pools.create_br(self.get_builder(), body_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), body_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);

        // Process body with break/continue targets, if it exists
        self.push_break_continue_target(end_block.clone(), cond_block.clone());
        if let Some(body_node) = body_node_opt {
            let _ = self.ir_router(body_node)?;
        }
        self.pop_target();

        // Branch to condition block
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in do-while!");
        resource_pools.create_br(self.get_builder(), cond_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), cond_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);

        // Process condition if it exists, otherwise use true (1) as default
        let llvm_cond = if let Some(cond_node) = cond_node_opt {
            let cond_ptr = self.ir_router(cond_node)?;
            let cond_ptr = cond_ptr.expect("Missing condition");
            match cond_ptr {
                Tag::Value(value) => value,
                _ => return Err(ErrorType::DevError { message: "Expected value tag".to_string() })
            }
        } else {
            // Default to true (1) if no condition provided
            let resource_pools = self.get_resource_pools();
            let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in do-while condition!");
            let true_value = resource_pools.create_integer(self.get_context(), 1)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant for true".to_string() })?;
            drop(resource_pools);
            true_value
        };

        // Re-acquire lock
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in do-while!");
        let zero = resource_pools.create_integer(self.get_context(), 0)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant".to_string() })?;
        let eq = resource_pools.build_icmp_eq(self.get_builder(), llvm_cond, zero, "cmptmp")
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create comparison".to_string() })?;
        let bool_cond = resource_pools.build_logical_not(self.get_builder(), self.get_context(), eq, "nottmp")
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create logical not".to_string() })?;

        // Create conditional branch
        resource_pools.create_cond_br(self.get_builder(), bool_cond, body_block, end_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create conditional branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), end_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;

        Ok(None)
    }

    /// Generates LLVM IR for a while loop.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a while loop.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the while.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a while loop from */
    /// //let result = self.generate_while_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_while_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let children = node.get_children();
        
        // Handle while loops with different numbers of children
        let (cond_node_opt, body_node_opt) = match children.len() {
            0 => (None, None), // Empty loop (just for completeness)
            1 => (Some(&children[0]), None), // Just condition
            2 => (Some(&children[0]), Some(&children[1])), // Condition and body
            _ => return Err(ErrorType::DevError { 
                message: format!("Invalid while node: unexpected number of children {}", children.len()) 
            })
        };

        let function = self.get_function().unwrap();
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in while!");

        let next_id = self.get_next_label_id();
        let cond_label = format!("while_condID{}", next_id);
        let body_label = format!("while_bodyID{}", next_id);
        let end_label = format!("while_endID{}", next_id);

        // Create blocks in the correct order
        let current_insert = self.get_current_insert_block().unwrap_or_else(|| {
            resource_pools.get_current_block(self.get_builder()).expect("No current block!")
        });
        let cond_block = resource_pools.create_basic_block_after(self.get_context(), function, current_insert, &cond_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create condition block".to_string() })?;
        let body_block = resource_pools.create_basic_block_after(self.get_context(), function, cond_block, &body_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create body block".to_string() })?;
        let end_block = resource_pools.create_basic_block_after(self.get_context(), function, body_block, &end_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create end block".to_string() })?;

        // Branch to condition block
        resource_pools.create_br(self.get_builder(), cond_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), cond_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);
        
        // Process condition if it exists, otherwise use true (1) as default
        let llvm_cond = if let Some(cond_node) = cond_node_opt {
            let cond_ptr = self.ir_router(cond_node)?;
            let cond_ptr = cond_ptr.expect("Missing condition");
            match cond_ptr {
                Tag::Value(value) => value,
                _ => return Err(ErrorType::DevError { message: "Expected value tag".to_string() })
            }
        } else {
            // Default to true (1) if no condition provided
            let resource_pools = self.get_resource_pools();
            let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in while condition!");
            let true_value = resource_pools.create_integer(self.get_context(), 1)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant for true".to_string() })?;
            drop(resource_pools);
            true_value
        };

        // Create conditional branch
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in while!");
        let zero = resource_pools.create_integer(self.get_context(), 0)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant".to_string() })?;
        let eq = resource_pools.build_icmp_eq(self.get_builder(), llvm_cond, zero, "cmptmp")
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create comparison".to_string() })?;
        let bool_cond = resource_pools.build_logical_not(self.get_builder(), self.get_context(), eq, "nottmp")
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create logical not".to_string() })?;
        resource_pools.create_cond_br(self.get_builder(), bool_cond, body_block.clone(), end_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create conditional branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), body_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);

        // Process body with break/continue targets, if it exists
        self.push_break_continue_target(end_block.clone(), cond_block.clone());
        if let Some(body_node) = body_node_opt {
            let _ = self.ir_router(body_node)?;
        }
        self.pop_target();

        // Branch back to condition block
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in while!");
        resource_pools.create_br(self.get_builder(), cond_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), end_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        Ok(None)
    }
    
    /// Generates LLVM IR for a for loop.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for a for loop.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the for loop.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate a for loop from */
    /// //let result = self.generate_for_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_for_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let children = node.get_children();
        
        // Handle for loops with different numbers of children
        let (init_node_opt, cond_node_opt, inc_node_opt, body_node_opt) = match children.len() {
            0 => (None, None, None, None), // Empty loop (just for completeness)
            1 => (None, None, None, Some(&children[0])), // Just body
            2 => (None, Some(&children[0]), None, Some(&children[1])), // Condition and body
            3 => (Some(&children[0]), Some(&children[1]), None, Some(&children[2])), // Init, condition, and body
            4 => (Some(&children[0]), Some(&children[1]), Some(&children[2]), Some(&children[3])), // All components
            _ => return Err(ErrorType::DevError { 
                message: format!("Invalid for node: unexpected number of children {}", children.len()) 
            })
        };
        
        // Process initialization if it exists
        if let Some(init_node) = init_node_opt {
            let _ = match init_node.get_node_type() {
                NodeType::LoopInitializer => {
                    if let Some(child) = init_node.get_children().first() {
                        self.ir_router(child)
                    } else {
                        Ok(None)
                    }
                },
                _ => self.ir_router(init_node)
            }?;
        }
        let function = self.get_function().unwrap();
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in for!");
        
        let next_id = self.get_next_label_id();
        let cond_label = format!("for_condID{}", next_id);
        let body_label = format!("for_bodyID{}", next_id);
        let inc_label = format!("for_incID{}", next_id);
        let end_label = format!("for_endID{}", next_id);
        
        // Create blocks in the correct order
        let current_insert = self.get_current_insert_block().unwrap_or_else(|| {
            resource_pools.get_current_block(self.get_builder()).expect("No current block!")
        });
        
        let cond_block = resource_pools.create_basic_block_after(self.get_context(), function, current_insert, &cond_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create condition block".to_string() })?;
        let body_block = resource_pools.create_basic_block_after(self.get_context(), function, cond_block, &body_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create body block".to_string() })?;
        let inc_block = resource_pools.create_basic_block_after(self.get_context(), function, body_block, &inc_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create increment block".to_string() })?;
        let end_block = resource_pools.create_basic_block_after(self.get_context(), function, inc_block, &end_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create end block".to_string() })?;
        
        // Branch to condition block
        resource_pools.create_br(self.get_builder(), cond_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), cond_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);
        // Process condition if it exists, otherwise use true (1) as default
        let llvm_cond = if let Some(cond_node) = cond_node_opt {
            let cond_ptr = self.ir_router(cond_node)?;
            let cond_ptr = cond_ptr.expect("Missing condition");
            match cond_ptr {
                Tag::Value(value) => value,
                _ => return Err(ErrorType::DevError { message: "Expected value tag".to_string() })
            }
        } else {
            // Default to true (1) if no condition provided
            let resource_pools = self.get_resource_pools();
            let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in for condition!");
            let true_value = resource_pools.create_integer(self.get_context(), 1)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant for true".to_string() })?;
            drop(resource_pools);
            true_value
        };
        
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in for!");
        let zero = resource_pools.create_integer(self.get_context(), 0)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant".to_string() })?;
        let eq = resource_pools.build_icmp_eq(self.get_builder(), llvm_cond, zero, "cmptmp")
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create comparison".to_string() })?;
        let bool_cond = resource_pools.build_logical_not(self.get_builder(), self.get_context(), eq, "nottmp")
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create logical not".to_string() })?;
        
        // Create conditional branch
        resource_pools.create_cond_br(self.get_builder(), bool_cond, body_block.clone(), end_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create conditional branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), body_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);
        // Process body with break/continue targets, if it exists
        self.push_break_continue_target(end_block.clone(), inc_block.clone());
        if let Some(body_node) = body_node_opt {
            let _ = self.ir_router(body_node)?;
        }
        self.pop_target();
        
        // Branch to increment block
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in for!");
        resource_pools.create_br(self.get_builder(), inc_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), inc_block.clone())
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);
        // Process increment if it exists
        if let Some(inc_node) = inc_node_opt {
            let _ = match inc_node.get_node_type() {
                NodeType::LoopIncrement => {
                    if let Some(child) = inc_node.get_children().first() {
                        self.ir_router(child)
                    } else {
                        Ok(None)
                    }
                },
                _ => self.ir_router(inc_node)
            }?;
        }
        
        // Branch back to condition block
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in for!");
        resource_pools.create_br(self.get_builder(), cond_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), end_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        Ok(None)
    }

    /// Generates LLVM IR for an if statement.
    /// 
    /// # Parameters
    ///
    /// - `node`: A reference to an `ASTNode` to generate IR for an if statement.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<Tag>, ErrorType>` containing None
    /// if generation went smoothly or an Error if there was a problem generating the if statement.
    ///
    /// # Errors
    ///
    /// - Returns an ErrorType if generation failed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// //let a_node: ASTNode = /* Some ASTNode we want to generate an if statement from */
    /// //let result = self.generate_for_ir(&a_node);
    /// /* check if type_result was Ok or Err, if Ok, it will contain None. */
    /// ```
    pub fn generate_if_ir(&mut self, node: &ASTNode) -> Result<Option<Tag>, ErrorType> {
        let children = node.get_children();
        if children.len() != 2 && children.len() != 3 {
            return Err(ErrorType::DevError { message: "Invalid if node".to_string() });
        }
        let cond_node = &children[0];
        let then_node = &children[1];
        let function = self.get_function().unwrap();
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in if!");
        let next_id = self.get_next_label_id();
        let then_label = format!("thenID{}", next_id);
        let else_label = format!("elseID{}", next_id);
        let merge_label = format!("mergeID{}", next_id);
        let current_insert = self.get_current_insert_block().unwrap_or_else(|| {
            resource_pools.get_current_block(self.get_builder()).expect("No current block!")
        });
        let then_block = resource_pools.create_basic_block_after(self.get_context(), function, current_insert, &then_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create then block".to_string() })?;
        let else_block = resource_pools.create_basic_block_after(self.get_context(), function, then_block, &else_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create else block".to_string() })?;
        let merge_block = resource_pools.create_basic_block_after(self.get_context(), function, else_block, &merge_label)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create merge block".to_string() })?;
        self.set_current_insert_block(merge_block);
        drop(resource_pools);
        let cond_ptr = self.ir_router(cond_node)?;
        let cond_ptr = cond_ptr.expect("Missing condition");
        let llvm_cond = match cond_ptr {
            Tag::Value(value) => value,
            _ => return Err(ErrorType::DevError { message: "Expected value tag".to_string() })
        };
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in if!");
        let bool_cond = if let NodeType::Literal(value) = cond_node.get_node_type() {
            if value == "true" || value == "false" {
                llvm_cond
            } else {
                let zero = resource_pools.create_integer(self.get_context(), 0)
                    .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant".to_string() })?;
                let eq = resource_pools.build_icmp_eq(self.get_builder(), llvm_cond, zero, "cmptmp")
                    .ok_or_else(|| ErrorType::DevError { message: "Failed to create comparison".to_string() })?;
                resource_pools.build_logical_not(self.get_builder(), self.get_context(), eq, "nottmp")
                    .ok_or_else(|| ErrorType::DevError { message: "Failed to create logical not".to_string() })?
            }
        } else {
            let zero = resource_pools.create_integer(self.get_context(), 0)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create integer constant".to_string() })?;
            let eq = resource_pools.build_icmp_eq(self.get_builder(), llvm_cond, zero, "cmptmp")
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create comparison".to_string() })?;
            resource_pools.build_logical_not(self.get_builder(), self.get_context(), eq, "nottmp")
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create logical not".to_string() })?
        };
        resource_pools.create_cond_br(self.get_builder(), bool_cond, then_block, else_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to create conditional branch".to_string() })?;
        resource_pools.position_builder_at_end(self.get_builder(), then_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);
        let then_result = self.ir_router(then_node)?;
        let has_return = matches!(then_result, Some(Tag::Value(_)));
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in if!");
        if !has_return {
            resource_pools.create_br(self.get_builder(), merge_block)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        }
        resource_pools.position_builder_at_end(self.get_builder(), else_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        drop(resource_pools);
        let else_result = if let Some(else_node) = children.get(2) {
            self.ir_router(else_node)?
        } else {
            None
        };
        let has_return = matches!(else_result, Some(Tag::Value(_)));
        let resource_pools = self.get_resource_pools();
        let mut resource_pools = resource_pools.try_lock().expect("Failed to lock mutex in if!");
        if !has_return {
            resource_pools.create_br(self.get_builder(), merge_block)
                .ok_or_else(|| ErrorType::DevError { message: "Failed to create branch".to_string() })?;
        }
        resource_pools.position_builder_at_end(self.get_builder(), merge_block)
            .ok_or_else(|| ErrorType::DevError { message: "Failed to position builder".to_string() })?;
        Ok(None)
    }
} 