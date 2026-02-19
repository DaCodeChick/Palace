//! Virtual Machine for Iptscrae script execution.
//!
//! The VM is a stack-based interpreter that executes Iptscrae AST nodes.
//! It maintains a value stack and variable storage, executing operations
//! by pushing/popping values from the stack.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::iptscrae::ast::{BinOp, Block, Expr, Script, Statement, UnaryOp};
use crate::iptscrae::context::{ScriptContext, SecurityLevel};
use crate::iptscrae::value::Value;

/// VM error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    /// Stack underflow - tried to pop from empty stack
    StackUnderflow { operation: String },
    /// Variable not found
    UndefinedVariable { name: String },
    /// Unknown function call
    UndefinedFunction { name: String },
    /// Type error (e.g., tried to add string to non-string)
    TypeError { message: String },
    /// Division by zero
    DivisionByZero,
    /// Break statement outside of loop
    BreakOutsideLoop,
    /// Execution timeout (for sandboxed scripts)
    Timeout,
    /// Instruction limit exceeded (for sandboxed scripts)
    InstructionLimitExceeded,
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::StackUnderflow { operation } => {
                write!(f, "Stack underflow during operation: {}", operation)
            }
            VmError::UndefinedVariable { name } => {
                write!(f, "Undefined variable: {}", name)
            }
            VmError::UndefinedFunction { name } => {
                write!(f, "Undefined function: {}", name)
            }
            VmError::TypeError { message } => {
                write!(f, "Type error: {}", message)
            }
            VmError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            VmError::BreakOutsideLoop => {
                write!(f, "Break statement outside of loop")
            }
            VmError::Timeout => {
                write!(f, "Script execution timeout")
            }
            VmError::InstructionLimitExceeded => {
                write!(f, "Instruction limit exceeded")
            }
        }
    }
}

impl std::error::Error for VmError {}

/// Control flow signals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlFlow {
    /// Normal execution
    Continue,
    /// Break out of loop
    Break,
}

/// VM execution limits for sandboxing
#[derive(Debug, Clone, Default)]
pub struct ExecutionLimits {
    max_instructions: Option<usize>,
    max_duration: Option<Duration>,
}

impl ExecutionLimits {
    /// Create limits for server scripts (no limits)
    pub const fn server() -> Self {
        Self {
            max_instructions: None,
            max_duration: None,
        }
    }

    /// Create limits for cyborg scripts (sandboxed)
    pub const fn cyborg() -> Self {
        Self {
            max_instructions: Some(100_000),
            max_duration: Some(Duration::from_secs(5)),
        }
    }

    /// Create custom limits with builder pattern
    pub const fn custom() -> Self {
        Self {
            max_instructions: None,
            max_duration: None,
        }
    }

    /// Set maximum number of instructions
    pub const fn with_max_instructions(mut self, max: usize) -> Self {
        self.max_instructions = Some(max);
        self
    }

    /// Set maximum execution duration
    pub const fn with_max_duration(mut self, duration: Duration) -> Self {
        self.max_duration = Some(duration);
        self
    }
}

/// Virtual Machine for executing Iptscrae scripts
pub struct Vm {
    /// Value stack
    stack: Vec<Value>,
    /// Variable storage
    variables: HashMap<String, Value>,
    /// Execution limits
    limits: ExecutionLimits,
    /// Instruction counter
    instruction_count: usize,
    /// Execution start time
    start_time: Option<Instant>,
    /// Output buffer (for SAY commands, etc.)
    output: Vec<String>,
}

impl Vm {
    /// Create a new VM with default (no) limits
    pub fn new() -> Self {
        Self::with_limits(ExecutionLimits::default())
    }

    /// Create a new VM with execution limits
    pub fn with_limits(limits: ExecutionLimits) -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
            limits,
            instruction_count: 0,
            start_time: None,
            output: Vec::new(),
        }
    }

    /// Execute a script
    pub fn execute(&mut self, _script: &Script) -> Result<(), VmError> {
        self.start_time = Some(Instant::now());
        self.instruction_count = 0;

        // Scripts don't have top-level code, only event handlers
        // Event handlers are executed separately when events occur
        Ok(())
    }

    /// Execute a specific event handler from a script with context
    pub fn execute_handler(
        &mut self,
        script: &Script,
        event_type: crate::iptscrae::events::EventType,
        context: &mut ScriptContext,
    ) -> Result<(), VmError> {
        self.start_time = Some(Instant::now());
        self.instruction_count = 0;

        // Find handlers matching the event type
        for handler in &script.handlers {
            if handler.event == event_type {
                self.execute_block_with_context(&handler.body, Some(context))?;
            }
        }

        Ok(())
    }

    /// Execute a block of statements with optional context
    fn execute_block_with_context(
        &mut self,
        block: &Block,
        mut context: Option<&mut ScriptContext>,
    ) -> Result<ControlFlow, VmError> {
        for statement in &block.statements {
            let flow = self.execute_statement_with_context(statement, context.as_deref_mut())?;
            if flow == ControlFlow::Break {
                return Ok(ControlFlow::Break);
            }
        }
        Ok(ControlFlow::Continue)
    }

    /// Execute a statement with optional context
    fn execute_statement_with_context(
        &mut self,
        statement: &Statement,
        mut context: Option<&mut ScriptContext>,
    ) -> Result<ControlFlow, VmError> {
        self.check_limits()?;

        match statement {
            Statement::Expr(expr) => {
                self.execute_expr_with_context(expr, context)?;
                Ok(ControlFlow::Continue)
            }

            Statement::Assign { name, .. } => {
                let value = self.pop("assignment")?;
                self.variables.insert(name.clone(), value);
                Ok(ControlFlow::Continue)
            }

            Statement::If {
                then_block,
                else_block,
                ..
            } => {
                // Condition was already evaluated and pushed to stack by parser
                let condition = self.pop("IF condition")?;

                if condition.to_bool() {
                    self.execute_block_with_context(then_block, context)?;
                } else if let Some(else_block) = else_block {
                    self.execute_block_with_context(else_block, context)?;
                }
                Ok(ControlFlow::Continue)
            }

            Statement::While { body, .. } => {
                loop {
                    // In Iptscrae, condition is re-evaluated each iteration
                    // For now, we need the condition to be evaluated before WHILE
                    // This is a simplified implementation
                    let condition = self.pop("WHILE condition")?;

                    if !condition.to_bool() {
                        break;
                    }

                    let flow = self.execute_block_with_context(body, context.as_deref_mut())?;
                    if flow == ControlFlow::Break {
                        break;
                    }
                }
                Ok(ControlFlow::Continue)
            }

            Statement::Break { .. } => Ok(ControlFlow::Break),
        }
    }

    /// Execute an expression with optional context
    fn execute_expr_with_context(
        &mut self,
        expr: &Expr,
        context: Option<&mut ScriptContext>,
    ) -> Result<(), VmError> {
        self.check_limits()?;

        match expr {
            Expr::Literal { value, .. } => {
                self.push(value.clone());
                Ok(())
            }

            Expr::Variable { name, .. } => {
                let value = self
                    .variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| VmError::UndefinedVariable { name: name.clone() })?;
                self.push(value);
                Ok(())
            }

            Expr::Call { name, .. } => {
                self.execute_builtin_with_context(name, context)?;
                Ok(())
            }

            Expr::BinOp { op, .. } => {
                self.execute_binop(*op)?;
                Ok(())
            }

            Expr::UnaryOp { op, .. } => {
                self.execute_unaryop(*op)?;
                Ok(())
            }

            Expr::Block(block) => {
                self.execute_block_with_context(block, context)?;
                Ok(())
            }
        }
    }

    /// Execute a block of statements
    /// Execute a binary operation
    fn execute_binop(&mut self, op: BinOp) -> Result<(), VmError> {
        // Pop operands (note: right operand is popped first due to stack order)
        let right = self.pop("binary operation right operand")?;
        let left = self.pop("binary operation left operand")?;

        let result = match op {
            BinOp::Add => Value::Integer(left.to_integer() + right.to_integer()),
            BinOp::Sub => Value::Integer(left.to_integer() - right.to_integer()),
            BinOp::Mul => Value::Integer(left.to_integer() * right.to_integer()),
            BinOp::Div => {
                let divisor = right.to_integer();
                if divisor == 0 {
                    return Err(VmError::DivisionByZero);
                }
                Value::Integer(left.to_integer() / divisor)
            }
            BinOp::Mod => {
                let divisor = right.to_integer();
                if divisor == 0 {
                    return Err(VmError::DivisionByZero);
                }
                Value::Integer(left.to_integer() % divisor)
            }
            BinOp::Concat => Value::String(format!("{}{}", left, right)),
            BinOp::Eq => Value::Integer(if left.to_integer() == right.to_integer() {
                1
            } else {
                0
            }),
            BinOp::NotEq => Value::Integer(if left.to_integer() != right.to_integer() {
                1
            } else {
                0
            }),
            BinOp::Less => Value::Integer(if left.to_integer() < right.to_integer() {
                1
            } else {
                0
            }),
            BinOp::Greater => Value::Integer(if left.to_integer() > right.to_integer() {
                1
            } else {
                0
            }),
            BinOp::LessEq => Value::Integer(if left.to_integer() <= right.to_integer() {
                1
            } else {
                0
            }),
            BinOp::GreaterEq => Value::Integer(if left.to_integer() >= right.to_integer() {
                1
            } else {
                0
            }),
            BinOp::And => Value::Integer(if left.to_bool() && right.to_bool() {
                1
            } else {
                0
            }),
            BinOp::Or => Value::Integer(if left.to_bool() || right.to_bool() {
                1
            } else {
                0
            }),
            BinOp::Xor => Value::Integer(if left.to_bool() != right.to_bool() {
                1
            } else {
                0
            }),
        };

        self.push(result);
        Ok(())
    }

    /// Execute a unary operation
    fn execute_unaryop(&mut self, op: UnaryOp) -> Result<(), VmError> {
        let operand = self.pop("unary operation")?;

        let result = match op {
            UnaryOp::Neg => Value::Integer(-operand.to_integer()),
            UnaryOp::Not => Value::Integer(if operand.to_bool() { 0 } else { 1 }),
        };

        self.push(result);
        Ok(())
    }

    /// Execute stack manipulation built-in functions
    fn execute_stack_builtin(&mut self, name: &str) -> Result<(), VmError> {
        match name {
            "DUP" => {
                let value = self.peek("DUP")?;
                self.push(value);
                Ok(())
            }
            "DROP" => {
                self.pop("DROP")?;
                Ok(())
            }
            "SWAP" => {
                let a = self.pop("SWAP first")?;
                let b = self.pop("SWAP second")?;
                self.push(a);
                self.push(b);
                Ok(())
            }
            "OVER" => {
                // Copy second value to top: a b -> a b a
                if self.stack.len() < 2 {
                    return Err(VmError::StackUnderflow {
                        operation: "OVER".to_string(),
                    });
                }
                let value = self.stack[self.stack.len() - 2].clone();
                self.push(value);
                Ok(())
            }
            "ROT" => {
                // Rotate top three: a b c -> b c a
                if self.stack.len() < 3 {
                    return Err(VmError::StackUnderflow {
                        operation: "ROT".to_string(),
                    });
                }
                let c = self.pop("ROT")?;
                let b = self.pop("ROT")?;
                let a = self.pop("ROT")?;
                self.push(b);
                self.push(c);
                self.push(a);
                Ok(())
            }
            "PICK" => {
                // Copy nth value to top (0-indexed from top)
                let n = self.pop("PICK")?.to_integer();
                if n < 0 {
                    return Err(VmError::TypeError {
                        message: "PICK index must be non-negative".to_string(),
                    });
                }
                let index = n as usize;
                if index >= self.stack.len() {
                    return Err(VmError::StackUnderflow {
                        operation: "PICK".to_string(),
                    });
                }
                let value = self.stack[self.stack.len() - 1 - index].clone();
                self.push(value);
                Ok(())
            }
            "POP" => {
                // Alias for DROP
                self.pop("POP")?;
                Ok(())
            }
            "STACKDEPTH" => {
                self.push(Value::Integer(self.stack.len() as i32));
                Ok(())
            }
            "TOPTYPE" => {
                let value = self.peek("TOPTYPE")?;
                let type_id = match value {
                    Value::Integer(_) => 1,
                    Value::String(_) => 2,
                    Value::Array(_) => 3,
                };
                self.push(Value::Integer(type_id));
                Ok(())
            }
            "VARTYPE" => {
                // Get type of variable - needs variable name from stack
                let var_name = self.pop("VARTYPE")?.to_string();
                if let Some(value) = self.variables.get(&var_name) {
                    let type_id = match value {
                        Value::Integer(_) => 1,
                        Value::String(_) => 2,
                        Value::Array(_) => 3,
                    };
                    self.push(Value::Integer(type_id));
                } else {
                    self.push(Value::Integer(0)); // Undefined = 0
                }
                Ok(())
            }
            _ => Err(VmError::UndefinedFunction {
                name: name.to_string(),
            }),
        }
    }

    /// Execute string manipulation built-in functions
    fn execute_string_builtin(&mut self, name: &str) -> Result<(), VmError> {
        match name {
            "ITOA" => {
                let value = self.pop("ITOA")?;
                self.push(Value::String(value.to_integer().to_string()));
                Ok(())
            }
            "ATOI" => {
                let value = self.pop("ATOI")?;
                self.push(Value::Integer(value.to_integer()));
                Ok(())
            }
            "STRLEN" => {
                let value = self.pop("STRLEN")?;
                self.push(Value::Integer(value.to_string().len() as i32));
                Ok(())
            }
            "UPPERCASE" => {
                let value = self.pop("UPPERCASE")?;
                self.push(Value::String(value.to_string().to_uppercase()));
                Ok(())
            }
            "LOWERCASE" => {
                let value = self.pop("LOWERCASE")?;
                self.push(Value::String(value.to_string().to_lowercase()));
                Ok(())
            }
            "SUBSTR" => {
                // Search for substring - pushes 1 if found, 0 if not
                let needle = self.pop("SUBSTR needle")?.to_string();
                let haystack = self.pop("SUBSTR haystack")?.to_string();
                let found = if haystack.contains(&needle) { 1 } else { 0 };
                self.push(Value::Integer(found));
                Ok(())
            }
            "SUBSTRING" => {
                // Extract substring: string start length -> substring
                let length = self.pop("SUBSTRING length")?.to_integer();
                let start = self.pop("SUBSTRING start")?.to_integer();
                let string = self.pop("SUBSTRING string")?.to_string();
                
                if start < 0 || length < 0 {
                    self.push(Value::String(String::new()));
                    return Ok(());
                }
                
                let start_idx = start as usize;
                let length_usize = length as usize;
                
                let result = string.chars()
                    .skip(start_idx)
                    .take(length_usize)
                    .collect::<String>();
                
                self.push(Value::String(result));
                Ok(())
            }
            "STRINDEX" => {
                // Find index of substring: haystack needle -> index (or -1 if not found)
                let needle = self.pop("STRINDEX needle")?.to_string();
                let haystack = self.pop("STRINDEX haystack")?.to_string();
                
                let index = haystack.find(&needle)
                    .map(|i| i as i32)
                    .unwrap_or(-1);
                
                self.push(Value::Integer(index));
                Ok(())
            }
            _ => Err(VmError::UndefinedFunction {
                name: name.to_string(),
            }),
        }
    }

    /// Execute Palace-specific built-in functions
    fn execute_palace_builtin(
        &mut self,
        name: &str,
        context: Option<&mut ScriptContext>,
    ) -> Result<(), VmError> {
        match name {
            "SAY" => {
                let message = self.pop("SAY")?;
                if let Some(ctx) = context {
                    ctx.actions.say(&message.to_string());
                } else {
                    // Fallback for tests
                    self.output.push(message.to_string());
                }
                Ok(())
            }
            "CHAT" => {
                let message = self.pop("CHAT")?;
                if let Some(ctx) = context {
                    ctx.actions.chat(&message.to_string());
                } else {
                    // Fallback for tests
                    self.output.push(message.to_string());
                }
                Ok(())
            }
            "LOCALMSG" => {
                let message = self.pop("LOCALMSG")?;
                if let Some(ctx) = context {
                    ctx.actions.local_msg(&message.to_string());
                }
                Ok(())
            }
            "ROOMMSG" => {
                let message = self.pop("ROOMMSG")?;
                if let Some(ctx) = context {
                    ctx.actions.room_msg(&message.to_string());
                }
                Ok(())
            }
            "PRIVATEMSG" => {
                let message = self.pop("PRIVATEMSG")?;
                let user_id = self.pop("PRIVATEMSG user_id")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.private_msg(user_id, &message.to_string());
                }
                Ok(())
            }
            "USERNAME" => {
                if let Some(ctx) = context {
                    self.push(Value::String(ctx.user_name.clone()));
                } else {
                    self.push(Value::String("Guest".to_string()));
                }
                Ok(())
            }
            "WHOME" => {
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.user_id));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "WHONAME" => {
                let user_id = self.pop("WHONAME")?.to_integer();
                if let Some(ctx) = context {
                    // Look up username by ID (would need context support)
                    // For now, just return user's own name if ID matches
                    if user_id == ctx.user_id {
                        self.push(Value::String(ctx.user_name.clone()));
                    } else {
                        self.push(Value::String(format!("User{}", user_id)));
                    }
                } else {
                    self.push(Value::String("Guest".to_string()));
                }
                Ok(())
            }
            "SETFACE" => {
                let face_id = self.pop("SETFACE")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.set_face(face_id as i16);
                }
                Ok(())
            }
            "SETCOLOR" => {
                let color = self.pop("SETCOLOR")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.set_color(color as i16);
                }
                Ok(())
            }
            "GETPROPS" => {
                if let Some(ctx) = context {
                    // Push number of props first, then each prop as crc:id
                    let num_props = ctx.user_props.len() as i32;
                    self.push(Value::Integer(num_props));
                    
                    // Push each prop's CRC and ID
                    for prop in &ctx.user_props {
                        self.push(Value::Integer(prop.crc as i32));
                        self.push(Value::Integer(prop.id));
                    }
                } else {
                    self.push(Value::Integer(0)); // No props
                }
                Ok(())
            }
            "SETPROPS" => {
                // Pop number of props
                let num_props = self.pop("SETPROPS num_props")?.to_integer();
                if num_props < 0 {
                    return Err(VmError::TypeError {
                        message: "SETPROPS num_props must be non-negative".to_string(),
                    });
                }
                
                let mut props = Vec::new();
                for _ in 0..num_props {
                    let id = self.pop("SETPROPS prop id")?.to_integer();
                    let crc = self.pop("SETPROPS prop crc")?.to_integer();
                    props.push(crate::AssetSpec {
                        id,
                        crc: crc as u32,
                    });
                }
                
                if let Some(ctx) = context {
                    ctx.actions.set_props(props);
                }
                Ok(())
            }
            // Prop manipulation functions
            "NAKED" => {
                // Remove all props
                if let Some(ctx) = context {
                    ctx.actions.set_props(Vec::new());
                }
                Ok(())
            }
            "DONPROP" => {
                // Add a prop: crc id -> add prop to user
                let id = self.pop("DONPROP id")?.to_integer();
                let crc = self.pop("DONPROP crc")?.to_integer();
                if let Some(ctx) = context {
                    let mut props = ctx.user_props.clone();
                    props.push(crate::AssetSpec {
                        id,
                        crc: crc as u32,
                    });
                    ctx.actions.set_props(props);
                }
                Ok(())
            }
            "DOFFPROP" => {
                // Remove a specific prop: id -> remove prop from user
                let id = self.pop("DOFFPROP id")?.to_integer();
                if let Some(ctx) = context {
                    let props: Vec<_> = ctx.user_props.iter()
                        .filter(|p| p.id != id)
                        .cloned()
                        .collect();
                    ctx.actions.set_props(props);
                }
                Ok(())
            }
            "DROPPROP" => {
                // Remove the last prop
                if let Some(ctx) = context {
                    let mut props = ctx.user_props.clone();
                    props.pop();
                    ctx.actions.set_props(props);
                }
                Ok(())
            }
            "REMOVEPROP" => {
                // Alias for DOFFPROP
                let id = self.pop("REMOVEPROP id")?.to_integer();
                if let Some(ctx) = context {
                    let props: Vec<_> = ctx.user_props.iter()
                        .filter(|p| p.id != id)
                        .cloned()
                        .collect();
                    ctx.actions.set_props(props);
                }
                Ok(())
            }
            "USERPROP" => {
                // Get prop at index: index -> crc id
                let index = self.pop("USERPROP")?.to_integer();
                if let Some(ctx) = context {
                    if index >= 0 && (index as usize) < ctx.user_props.len() {
                        let prop = &ctx.user_props[index as usize];
                        self.push(Value::Integer(prop.crc as i32));
                        self.push(Value::Integer(prop.id));
                    } else {
                        self.push(Value::Integer(0));
                        self.push(Value::Integer(0));
                    }
                } else {
                    self.push(Value::Integer(0));
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "NBRUSERPROPS" => {
                // Get number of props user is wearing
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.user_props.len() as i32));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "TOPPROP" => {
                // Get the top (last) prop: -> crc id
                if let Some(ctx) = context {
                    if let Some(prop) = ctx.user_props.last() {
                        self.push(Value::Integer(prop.crc as i32));
                        self.push(Value::Integer(prop.id));
                    } else {
                        self.push(Value::Integer(0));
                        self.push(Value::Integer(0));
                    }
                } else {
                    self.push(Value::Integer(0));
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "HASPROP" => {
                // Check if user has a specific prop: id -> boolean
                let id = self.pop("HASPROP")?.to_integer();
                if let Some(ctx) = context {
                    let has_prop = ctx.user_props.iter().any(|p| p.id == id);
                    self.push(Value::Integer(if has_prop { 1 } else { 0 }));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "MACRO" => {
                // Execute a macro (prop script) - stub for now
                let _macro_id = self.pop("MACRO")?.to_integer();
                // Would need to look up and execute the macro script
                Ok(())
            }
            "ROOMNAME" => {
                if let Some(ctx) = context {
                    self.push(Value::String(ctx.room_name.clone()));
                } else {
                    self.push(Value::String(String::new()));
                }
                Ok(())
            }
            "ROOMID" => {
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.room_id as i32));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "GOTOROOM" => {
                let room_id = self.pop("GOTOROOM")?.to_integer();
                if let Some(ctx) = context {
                    if !ctx.is_function_allowed("GOTOROOM") {
                        return Err(VmError::TypeError {
                            message: "GOTOROOM not allowed at this security level".to_string(),
                        });
                    }
                    ctx.actions.goto_room(room_id as i16);
                }
                Ok(())
            }
            "LOCK" => {
                let door_id = self.pop("LOCK")?.to_integer();
                if let Some(ctx) = context {
                    if !ctx.is_function_allowed("LOCK") {
                        return Err(VmError::TypeError {
                            message: "LOCK not allowed at this security level".to_string(),
                        });
                    }
                    ctx.actions.lock_door(door_id);
                }
                Ok(())
            }
            "UNLOCK" => {
                let door_id = self.pop("UNLOCK")?.to_integer();
                if let Some(ctx) = context {
                    if !ctx.is_function_allowed("UNLOCK") {
                        return Err(VmError::TypeError {
                            message: "UNLOCK not allowed at this security level".to_string(),
                        });
                    }
                    ctx.actions.unlock_door(door_id);
                }
                Ok(())
            }
            // Position/Movement functions
            "POSX" => {
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.user_pos_x as i32));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "POSY" => {
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.user_pos_y as i32));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "SETPOS" => {
                let y = self.pop("SETPOS y")?.to_integer();
                let x = self.pop("SETPOS x")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.set_pos(x as i16, y as i16);
                    ctx.user_pos_x = x as i16;
                    ctx.user_pos_y = y as i16;
                }
                Ok(())
            }
            "MOVE" => {
                let dy = self.pop("MOVE dy")?.to_integer();
                let dx = self.pop("MOVE dx")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.move_user(dx as i16, dy as i16);
                    ctx.user_pos_x += dx as i16;
                    ctx.user_pos_y += dy as i16;
                }
                Ok(())
            }
            "WHOPOS" => {
                let user_id = self.pop("WHOPOS")?.to_integer();
                // For now, return current user's position if ID matches
                if let Some(ctx) = context {
                    if user_id == ctx.user_id {
                        self.push(Value::Integer(ctx.user_pos_x as i32));
                        self.push(Value::Integer(ctx.user_pos_y as i32));
                    } else {
                        // Would need to look up other user's position
                        self.push(Value::Integer(0));
                        self.push(Value::Integer(0));
                    }
                } else {
                    self.push(Value::Integer(0));
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            // Room/Navigation functions
            "GOTOURL" => {
                let url = self.pop("GOTOURL")?.to_string();
                if let Some(ctx) = context {
                    ctx.actions.goto_url(&url);
                }
                Ok(())
            }
            "GOTOURLFRAME" => {
                let frame = self.pop("GOTOURLFRAME frame")?.to_string();
                let url = self.pop("GOTOURLFRAME url")?.to_string();
                if let Some(ctx) = context {
                    ctx.actions.goto_url_frame(&url, &frame);
                }
                Ok(())
            }
            "NETGOTO" => {
                // NETGOTO: server_name room_id -> navigate to room on another server
                let room_id = self.pop("NETGOTO room_id")?.to_integer();
                let server = self.pop("NETGOTO server")?.to_string();
                // Construct URL and delegate to GOTOURL
                let url = format!("palace://{}?room={}", server, room_id);
                if let Some(ctx) = context {
                    ctx.actions.goto_url(&url);
                }
                Ok(())
            }
            "NBRROOMUSERS" => {
                // Number of users in current room - would need room state
                // For now, return 1 (just the current user)
                self.push(Value::Integer(1));
                Ok(())
            }
            "ROOMUSER" => {
                // Get user ID by index in room - would need room state
                let index = self.pop("ROOMUSER")?.to_integer();
                if let Some(ctx) = context {
                    if index == 0 {
                        self.push(Value::Integer(ctx.user_id));
                    } else {
                        self.push(Value::Integer(0));
                    }
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            // User Info functions
            "USERID" => {
                // USERID is an alias for WHOME
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.user_id));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "WHOCHAT" => {
                // Get user ID from last chat message - would need event data
                if let Some(ctx) = context {
                    if let Some(Value::Integer(user_id)) = ctx.event_data.get("chat_user_id") {
                        self.push(Value::Integer(*user_id));
                    } else {
                        self.push(Value::Integer(ctx.user_id));
                    }
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "WHOTARGET" => {
                // Get targeted user ID - would need event data
                if let Some(ctx) = context {
                    if let Some(Value::Integer(user_id)) = ctx.event_data.get("target_user_id") {
                        self.push(Value::Integer(*user_id));
                    } else {
                        self.push(Value::Integer(0));
                    }
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "ISGOD" => {
                // Check if current user has god/wizard privileges
                if let Some(ctx) = context {
                    let is_god = matches!(ctx.security_level, SecurityLevel::Admin);
                    self.push(Value::Integer(if is_god { 1 } else { 0 }));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "ISWIZARD" => {
                // Alias for ISGOD
                if let Some(ctx) = context {
                    let is_wizard = matches!(ctx.security_level, SecurityLevel::Admin);
                    self.push(Value::Integer(if is_wizard { 1 } else { 0 }));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "ISGUEST" => {
                // Check if user is a guest (would need user flags)
                // For now, return 0 (not a guest)
                self.push(Value::Integer(0));
                Ok(())
            }
            "KILLUSER" => {
                // Disconnect a user - admin only
                let user_id = self.pop("KILLUSER")?.to_integer();
                if let Some(ctx) = context {
                    if !matches!(ctx.security_level, SecurityLevel::Admin) {
                        return Err(VmError::TypeError {
                            message: "KILLUSER requires admin privileges".to_string(),
                        });
                    }
                    // Would need server action to disconnect user
                    // For now, just consume the parameter
                    let _ = user_id;
                }
                Ok(())
            }
            // Message functions
            "SAYAT" => {
                let y = self.pop("SAYAT y")?.to_integer();
                let x = self.pop("SAYAT x")?.to_integer();
                let message = self.pop("SAYAT message")?.to_string();
                if let Some(ctx) = context {
                    // SAYAT displays text at a specific position
                    // Format as special message with coordinates
                    let msg = format!("@{},{}: {}", x, y, message);
                    ctx.actions.say(&msg);
                }
                Ok(())
            }
            "GLOBALMSG" => {
                let message = self.pop("GLOBALMSG")?.to_string();
                if let Some(ctx) = context {
                    ctx.actions.global_msg(&message);
                }
                Ok(())
            }
            "STATUSMSG" => {
                let message = self.pop("STATUSMSG")?.to_string();
                if let Some(ctx) = context {
                    ctx.actions.status_msg(&message);
                }
                Ok(())
            }
            "SUSRMSG" => {
                let message = self.pop("SUSRMSG")?.to_string();
                if let Some(ctx) = context {
                    if !matches!(ctx.security_level, SecurityLevel::Admin) {
                        return Err(VmError::TypeError {
                            message: "SUSRMSG requires admin privileges".to_string(),
                        });
                    }
                    ctx.actions.superuser_msg(&message);
                }
                Ok(())
            }
            "LOGMSG" => {
                let message = self.pop("LOGMSG")?.to_string();
                if let Some(ctx) = context {
                    ctx.actions.log_msg(&message);
                }
                Ok(())
            }
            // Door Commands
            "DOORIDX" => {
                // Get current door index - would need event data
                if let Some(ctx) = context {
                    if let Some(Value::Integer(door_id)) = ctx.event_data.get("door_id") {
                        self.push(Value::Integer(*door_id));
                    } else {
                        self.push(Value::Integer(-1));
                    }
                } else {
                    self.push(Value::Integer(-1));
                }
                Ok(())
            }
            "NBRDOORS" => {
                // Get number of doors in room - would need room data
                // For now, return 0
                self.push(Value::Integer(0));
                Ok(())
            }
            "DEST" => {
                // Get destination room ID for a door - would need room data
                let _door_id = self.pop("DEST")?.to_integer();
                // For now, return 0
                self.push(Value::Integer(0));
                Ok(())
            }
            "ISLOCKED" => {
                // Check if door is locked - would need room state
                let _door_id = self.pop("ISLOCKED")?.to_integer();
                // For now, return 0 (unlocked)
                self.push(Value::Integer(0));
                Ok(())
            }
            // Spot Commands
            "SPOTIDX" => {
                // Get current spot index - would need event data
                if let Some(ctx) = context {
                    if let Some(Value::Integer(spot_id)) = ctx.event_data.get("spot_id") {
                        self.push(Value::Integer(*spot_id));
                    } else {
                        self.push(Value::Integer(-1));
                    }
                } else {
                    self.push(Value::Integer(-1));
                }
                Ok(())
            }
            "NBRSPOTS" => {
                // Get number of spots in room - would need room data
                // For now, return 0
                self.push(Value::Integer(0));
                Ok(())
            }
            "SPOTNAME" => {
                // Get name of spot by ID - would need room data
                let _spot_id = self.pop("SPOTNAME")?.to_integer();
                self.push(Value::String(String::new()));
                Ok(())
            }
            "SPOTDEST" => {
                // Get destination for spot - would need room data
                let _spot_id = self.pop("SPOTDEST")?.to_integer();
                // Returns room_id
                self.push(Value::Integer(0));
                Ok(())
            }
            "INSPOT" => {
                // Check if user is in a specific spot - would need position/spot data
                let _spot_id = self.pop("INSPOT")?.to_integer();
                // For now, return 0 (not in spot)
                self.push(Value::Integer(0));
                Ok(())
            }
            "GETSPOTSTATE" => {
                // Get state of a spot
                let _spot_id = self.pop("GETSPOTSTATE")?.to_integer();
                // For now, return 0
                self.push(Value::Integer(0));
                Ok(())
            }
            "SETSPOTSTATE" => {
                // Set state of a spot (global)
                let state = self.pop("SETSPOTSTATE state")?.to_integer();
                let spot_id = self.pop("SETSPOTSTATE spot_id")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.set_spot_state(spot_id, state);
                }
                Ok(())
            }
            "SETSPOTSTATELOCAL" => {
                // Set state of a spot (local to user)
                let _state = self.pop("SETSPOTSTATELOCAL state")?.to_integer();
                let _spot_id = self.pop("SETSPOTSTATELOCAL spot_id")?.to_integer();
                // Would need local state storage
                Ok(())
            }
            "SETLOC" => {
                // Set location (alias for SETPOS)
                let y = self.pop("SETLOC y")?.to_integer();
                let x = self.pop("SETLOC x")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.set_pos(x as i16, y as i16);
                    ctx.user_pos_x = x as i16;
                    ctx.user_pos_y = y as i16;
                }
                Ok(())
            }
            "SETPICLOC" => {
                // Set picture location (for drawing) - would need graphics context
                let _y = self.pop("SETPICLOC y")?.to_integer();
                let _x = self.pop("SETPICLOC x")?.to_integer();
                // For now, just consume the parameters
                Ok(())
            }
            // Loose Props
            "ADDLOOSEPROP" => {
                // Add a loose prop to the room
                let y = self.pop("ADDLOOSEPROP y")?.to_integer();
                let x = self.pop("ADDLOOSEPROP x")?.to_integer();
                let prop_id = self.pop("ADDLOOSEPROP prop_id")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.add_loose_prop(prop_id, x as i16, y as i16);
                }
                Ok(())
            }
            "CLEARLOOSEPROPS" => {
                // Clear all loose props from room
                if let Some(ctx) = context {
                    ctx.actions.clear_loose_props();
                }
                Ok(())
            }
            "SHOWLOOSEPROPS" => {
                // Show/hide loose props - would need UI state
                let _show = self.pop("SHOWLOOSEPROPS")?.to_integer();
                // For now, just consume the parameter
                Ok(())
            }
            // Server/System functions
            "SERVERNAME" => {
                if let Some(ctx) = context {
                    self.push(Value::String(ctx.server_name.clone()));
                } else {
                    self.push(Value::String("localhost".to_string()));
                }
                Ok(())
            }
            "CLIENTTYPE" => {
                // Return client type identifier
                self.push(Value::String("Palace".to_string()));
                Ok(())
            }
            "IPTVERSION" => {
                // Return Iptscrae version
                self.push(Value::Integer(1));
                Ok(())
            }
            "DATETIME" => {
                // Return current datetime as string
                // Format: "MM/DD/YYYY HH:MM:SS"
                use std::time::SystemTime;
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                self.push(Value::String(format!("{}", now)));
                Ok(())
            }
            "TICKS" => {
                // Return ticks (milliseconds since start)
                use std::time::SystemTime;
                let ticks = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i32;
                self.push(Value::Integer(ticks));
                Ok(())
            }
            "MOUSEPOS" => {
                // Get mouse position - would need UI state
                // Push X and Y coordinates
                self.push(Value::Integer(0));
                self.push(Value::Integer(0));
                Ok(())
            }
            "DELAY" => {
                // Delay execution - not implemented (would need async/timer support)
                let _milliseconds = self.pop("DELAY")?.to_integer();
                Ok(())
            }
            "DIMROOM" => {
                // Dim the room - UI effect
                let _brightness = self.pop("DIMROOM")?.to_integer();
                Ok(())
            }
            "GLOBAL" => {
                // Access global variable - would need global variable storage
                let var_name = self.pop("GLOBAL")?.to_string();
                // For now, treat as regular variable
                if let Some(value) = self.variables.get(&var_name) {
                    self.push(value.clone());
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            // Special functions
            "ME" => {
                // Return current user ID
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.user_id));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            "ID" => {
                // Alias for ME
                if let Some(ctx) = context {
                    self.push(Value::Integer(ctx.user_id));
                } else {
                    self.push(Value::Integer(0));
                }
                Ok(())
            }
            // Sound/Media functions
            "SOUND" => {
                let sound_id = self.pop("SOUND")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.play_sound(sound_id);
                }
                Ok(())
            }
            "MIDIPLAY" => {
                let midi_id = self.pop("MIDIPLAY")?.to_integer();
                if let Some(ctx) = context {
                    ctx.actions.play_midi(midi_id);
                }
                Ok(())
            }
            "MIDISTOP" => {
                if let Some(ctx) = context {
                    ctx.actions.stop_midi();
                }
                Ok(())
            }
            "BEEP" => {
                if let Some(ctx) = context {
                    ctx.actions.beep();
                }
                Ok(())
            }
            "LAUNCHAPP" => {
                let url = self.pop("LAUNCHAPP")?.to_string();
                if let Some(ctx) = context {
                    ctx.actions.launch_app(&url);
                }
                Ok(())
            }
            // Painting/Graphics functions (stubs - would need graphics context)
            "LINE" => {
                // Draw line: x1 y1 x2 y2 -> draw line
                let _y2 = self.pop("LINE y2")?.to_integer();
                let _x2 = self.pop("LINE x2")?.to_integer();
                let _y1 = self.pop("LINE y1")?.to_integer();
                let _x1 = self.pop("LINE x1")?.to_integer();
                Ok(())
            }
            "LINETO" => {
                // Draw line to: x y -> draw line from pen position to x,y
                let _y = self.pop("LINETO y")?.to_integer();
                let _x = self.pop("LINETO x")?.to_integer();
                Ok(())
            }
            "PENPOS" => {
                // Get pen position - push X and Y
                self.push(Value::Integer(0));
                self.push(Value::Integer(0));
                Ok(())
            }
            "PENTO" => {
                // Set pen position
                let _y = self.pop("PENTO y")?.to_integer();
                let _x = self.pop("PENTO x")?.to_integer();
                Ok(())
            }
            "PENSIZE" => {
                // Set pen size
                let _size = self.pop("PENSIZE")?.to_integer();
                Ok(())
            }
            "PENCOLOR" => {
                // Set pen color
                let _color = self.pop("PENCOLOR")?.to_integer();
                Ok(())
            }
            "PENFRONT" => {
                // Set pen to draw in front
                Ok(())
            }
            "PENBACK" => {
                // Set pen to draw in back
                Ok(())
            }
            "PAINTCLEAR" => {
                // Clear all painting
                Ok(())
            }
            "PAINTUNDO" => {
                // Undo last painting operation
                Ok(())
            }
            _ => Err(VmError::UndefinedFunction {
                name: name.to_string(),
            }),
        }
    }

    /// Execute math built-in functions
    fn execute_math_builtin(&mut self, name: &str) -> Result<(), VmError> {
        match name {
            "RANDOM" => {
                // RANDOM takes max value from stack, returns random 0..max
                let max = self.pop("RANDOM")?.to_integer();
                if max <= 0 {
                    self.push(Value::Integer(0));
                } else {
                    // Simple pseudo-random using instruction count as seed
                    let random_val = (self.instruction_count as i32 * 1103515245 + 12345) % max;
                    self.push(Value::Integer(random_val.abs()));
                }
                Ok(())
            }
            "SQUAREROOT" => {
                let value = self.pop("SQUAREROOT")?.to_integer();
                let result = if value >= 0 {
                    (value as f64).sqrt() as i32
                } else {
                    0
                };
                self.push(Value::Integer(result));
                Ok(())
            }
            "SINE" => {
                // Sine in degrees * 1000
                let degrees = self.pop("SINE")?.to_integer();
                let radians = (degrees as f64).to_radians();
                let result = (radians.sin() * 1000.0) as i32;
                self.push(Value::Integer(result));
                Ok(())
            }
            "COSINE" => {
                // Cosine in degrees * 1000
                let degrees = self.pop("COSINE")?.to_integer();
                let radians = (degrees as f64).to_radians();
                let result = (radians.cos() * 1000.0) as i32;
                self.push(Value::Integer(result));
                Ok(())
            }
            "TANGENT" => {
                // Tangent in degrees * 1000
                let degrees = self.pop("TANGENT")?.to_integer();
                let radians = (degrees as f64).to_radians();
                let result = (radians.tan() * 1000.0) as i32;
                self.push(Value::Integer(result));
                Ok(())
            }
            _ => Err(VmError::UndefinedFunction {
                name: name.to_string(),
            }),
        }
    }

    /// Execute array built-in functions
    fn execute_array_builtin(&mut self, name: &str) -> Result<(), VmError> {
        match name {
            "ARRAY" => {
                // ARRAY takes size from stack, creates array of that size
                let size = self.pop("ARRAY")?.to_integer();
                if size < 0 {
                    return Err(VmError::TypeError {
                        message: "ARRAY size must be non-negative".to_string(),
                    });
                }
                let arr = vec![Value::Integer(0); size as usize];
                self.push(Value::Array(arr));
                Ok(())
            }
            "GET" => {
                // GET: array index -> value
                let index = self.pop("GET index")?.to_integer();
                let array = self.pop("GET array")?;
                
                if let Some(arr) = array.as_array() {
                    if index < 0 || index >= arr.len() as i32 {
                        return Err(VmError::TypeError {
                            message: format!("Array index {} out of bounds", index),
                        });
                    }
                    self.push(arr[index as usize].clone());
                } else {
                    return Err(VmError::TypeError {
                        message: "GET requires an array".to_string(),
                    });
                }
                Ok(())
            }
            "PUT" => {
                // PUT: array index value -> array (modified)
                let value = self.pop("PUT value")?;
                let index = self.pop("PUT index")?.to_integer();
                let mut array = self.pop("PUT array")?;
                
                if let Some(arr) = array.as_array_mut() {
                    if index < 0 || index >= arr.len() as i32 {
                        return Err(VmError::TypeError {
                            message: format!("Array index {} out of bounds", index),
                        });
                    }
                    arr[index as usize] = value;
                    self.push(array);
                } else {
                    return Err(VmError::TypeError {
                        message: "PUT requires an array".to_string(),
                    });
                }
                Ok(())
            }
            "LENGTH" => {
                // LENGTH: array -> length (also works on strings)
                let value = self.pop("LENGTH")?;
                let length = match value {
                    Value::Array(ref arr) => arr.len() as i32,
                    Value::String(ref s) => s.len() as i32,
                    Value::Integer(_) => 0,
                };
                self.push(Value::Integer(length));
                Ok(())
            }
            _ => Err(VmError::UndefinedFunction {
                name: name.to_string(),
            }),
        }
    }

    /// Execute a built-in function with optional context
    fn execute_builtin_with_context(
        &mut self,
        name: &str,
        context: Option<&mut ScriptContext>,
    ) -> Result<(), VmError> {
        let name_upper = name.to_uppercase();
        let name_str = name_upper.as_str();

        // Try stack operations first (most common)
        match self.execute_stack_builtin(name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try string operations
        match self.execute_string_builtin(name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try math operations
        match self.execute_math_builtin(name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try array operations
        match self.execute_array_builtin(name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try Palace operations
        self.execute_palace_builtin(name_str, context)
    }

    /// Push a value onto the stack
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop a value from the stack
    fn pop(&mut self, operation: &str) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| VmError::StackUnderflow {
            operation: operation.to_string(),
        })
    }

    /// Peek at top value without removing it
    fn peek(&self, operation: &str) -> Result<Value, VmError> {
        self.stack
            .last()
            .cloned()
            .ok_or_else(|| VmError::StackUnderflow {
                operation: operation.to_string(),
            })
    }

    /// Check execution limits
    fn check_limits(&mut self) -> Result<(), VmError> {
        self.instruction_count += 1;

        // Check instruction limit
        if let Some(max_instructions) = self.limits.max_instructions
            && self.instruction_count >= max_instructions
        {
            return Err(VmError::InstructionLimitExceeded);
        }

        // Check time limit
        if let Some(max_duration) = self.limits.max_duration
            && let Some(start) = self.start_time
            && start.elapsed() >= max_duration
        {
            return Err(VmError::Timeout);
        }

        Ok(())
    }

    /// Get the current stack (for debugging)
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    /// Get a variable value
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Get output buffer
    pub fn output(&self) -> &[String] {
        &self.output
    }

    /// Clear output buffer
    pub fn clear_output(&mut self) {
        self.output.clear();
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iptscrae::{Lexer, Parser};

    #[allow(dead_code)]
    fn parse_and_execute(source: &str) -> Result<Vm, VmError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();

        let mut vm = Vm::new();
        vm.execute(&script)?;
        Ok(vm)
    }

    #[test]
    fn test_vm_push_pop() {
        let mut vm = Vm::new();
        vm.push(Value::Integer(42));
        vm.push(Value::String("test".to_string()));

        assert_eq!(vm.pop("test").unwrap(), Value::String("test".to_string()));
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_vm_stack_underflow() {
        let mut vm = Vm::new();
        let result = vm.pop("test");
        assert!(matches!(result, Err(VmError::StackUnderflow { .. })));
    }

    #[test]
    fn test_vm_arithmetic() {
        let mut vm = Vm::new();

        // 5 + 3
        vm.push(Value::Integer(5));
        vm.push(Value::Integer(3));
        vm.execute_binop(BinOp::Add).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(8));

        // 10 - 4
        vm.push(Value::Integer(10));
        vm.push(Value::Integer(4));
        vm.execute_binop(BinOp::Sub).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(6));

        // 6 * 7
        vm.push(Value::Integer(6));
        vm.push(Value::Integer(7));
        vm.execute_binop(BinOp::Mul).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(42));

        // 15 / 3
        vm.push(Value::Integer(15));
        vm.push(Value::Integer(3));
        vm.execute_binop(BinOp::Div).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_vm_division_by_zero() {
        let mut vm = Vm::new();
        vm.push(Value::Integer(10));
        vm.push(Value::Integer(0));
        let result = vm.execute_binop(BinOp::Div);
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_vm_comparison() {
        let mut vm = Vm::new();

        // 5 < 10
        vm.push(Value::Integer(5));
        vm.push(Value::Integer(10));
        vm.execute_binop(BinOp::Less).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));

        // 10 > 5
        vm.push(Value::Integer(10));
        vm.push(Value::Integer(5));
        vm.execute_binop(BinOp::Greater).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));

        // 5 == 5
        vm.push(Value::Integer(5));
        vm.push(Value::Integer(5));
        vm.execute_binop(BinOp::Eq).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));
    }

    #[test]
    fn test_vm_string_concat() {
        let mut vm = Vm::new();
        vm.push(Value::String("Hello ".to_string()));
        vm.push(Value::String("World".to_string()));
        vm.execute_binop(BinOp::Concat).unwrap();
        assert_eq!(
            vm.pop("test").unwrap(),
            Value::String("Hello World".to_string())
        );
    }

    #[test]
    fn test_vm_variables() {
        let mut vm = Vm::new();
        vm.set_variable("x".to_string(), Value::Integer(42));
        assert_eq!(vm.get_variable("x"), Some(&Value::Integer(42)));
        assert_eq!(vm.get_variable("y"), None);
    }

    #[test]
    fn test_vm_stack_operations() {
        let mut vm = Vm::new();

        // Test DUP
        vm.push(Value::Integer(42));
        vm.execute_builtin_with_context("DUP", None).unwrap();
        assert_eq!(vm.stack.len(), 2);
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(42));
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(42));

        // Test SWAP
        vm.push(Value::Integer(1));
        vm.push(Value::Integer(2));
        vm.execute_builtin_with_context("SWAP", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_vm_string_operations() {
        let mut vm = Vm::new();

        // ITOA
        vm.push(Value::Integer(42));
        vm.execute_builtin_with_context("ITOA", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::String("42".to_string()));

        // ATOI
        vm.push(Value::String("123".to_string()));
        vm.execute_builtin_with_context("ATOI", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(123));
    }

    #[test]
    fn test_vm_say_command() {
        let mut vm = Vm::new();
        vm.push(Value::String("Hello World".to_string()));
        vm.execute_builtin_with_context("SAY", None).unwrap();
        assert_eq!(vm.output(), &["Hello World"]);
    }

    #[test]
    fn test_vm_execution_limits_instructions() {
        let limits = ExecutionLimits::custom().with_max_instructions(10);
        let mut vm = Vm::with_limits(limits);
        vm.start_time = Some(Instant::now());

        // Execute 11 instructions should fail
        for _ in 0..11 {
            let result = vm.check_limits();
            if result.is_err() {
                assert!(matches!(result, Err(VmError::InstructionLimitExceeded)));
                return;
            }
        }
        panic!("Should have hit instruction limit");
    }

    #[test]
    fn test_vm_new_builtins() {
        let mut vm = Vm::new();

        // Test PICK
        vm.push(Value::Integer(1));
        vm.push(Value::Integer(2));
        vm.push(Value::Integer(3));
        vm.push(Value::Integer(1)); // Pick index 1 (should get value 2)
        vm.execute_builtin_with_context("PICK", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(2));

        // Test STRLEN
        vm.push(Value::String("hello".to_string()));
        vm.execute_builtin_with_context("STRLEN", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(5));

        // Test UPPERCASE
        vm.push(Value::String("hello".to_string()));
        vm.execute_builtin_with_context("UPPERCASE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::String("HELLO".to_string()));

        // Test LOWERCASE
        vm.push(Value::String("WORLD".to_string()));
        vm.execute_builtin_with_context("LOWERCASE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::String("world".to_string()));
    }

    #[test]
    fn test_vm_integration_greeting() {
        use crate::iptscrae::{EventType, Lexer, Parser, ScriptActions, ScriptContext, SecurityLevel};
        use crate::AssetSpec;

        // Test action handler that captures SAY output
        struct TestActions {
            output: Vec<String>,
        }
        impl ScriptActions for TestActions {
            fn say(&mut self, message: &str) {
                self.output.push(message.to_string());
            }
            fn chat(&mut self, _message: &str) {}
            fn local_msg(&mut self, _message: &str) {}
            fn room_msg(&mut self, _message: &str) {}
            fn private_msg(&mut self, _user_id: i32, _message: &str) {}
            fn goto_room(&mut self, _room_id: i16) {}
            fn lock_door(&mut self, _door_id: i32) {}
            fn unlock_door(&mut self, _door_id: i32) {}
            fn set_face(&mut self, _face_id: i16) {}
            fn set_color(&mut self, _color: i16) {}
            fn set_props(&mut self, _props: Vec<AssetSpec>) {}
            fn set_pos(&mut self, _x: i16, _y: i16) {}
            fn move_user(&mut self, _dx: i16, _dy: i16) {}
            fn goto_url(&mut self, _url: &str) {}
            fn goto_url_frame(&mut self, _url: &str, _frame: &str) {}
            fn global_msg(&mut self, _message: &str) {}
            fn status_msg(&mut self, _message: &str) {}
            fn superuser_msg(&mut self, _message: &str) {}
            fn log_msg(&mut self, _message: &str) {}
            fn set_spot_state(&mut self, _spot_id: i32, _state: i32) {}
            fn add_loose_prop(&mut self, _prop_id: i32, _x: i16, _y: i16) {}
            fn clear_loose_props(&mut self) {}
            fn play_sound(&mut self, _sound_id: i32) {}
            fn play_midi(&mut self, _midi_id: i32) {}
            fn stop_midi(&mut self) {}
            fn beep(&mut self) {}
            fn launch_app(&mut self, _url: &str) {}
        }

        // Test a simple greeting script
        let source = r#"
            ON ENTER {
                USERNAME " has entered!" & SAY
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();

        let mut actions = TestActions {
            output: Vec::new(),
        };
        {
            let mut context = ScriptContext::new(SecurityLevel::Server, &mut actions);
            context.user_name = "Alice".to_string();
            context.event_type = EventType::Enter;

            let mut vm = Vm::new();
            vm.execute_handler(&script, EventType::Enter, &mut context)
                .unwrap();
        }

        assert_eq!(actions.output, vec!["Alice has entered!"]);
    }

    #[test]
    fn test_vm_integration_counter() {
        use crate::iptscrae::{EventType, Lexer, Parser, ScriptActions, ScriptContext, SecurityLevel};
        use crate::AssetSpec;

        // Test action handler that captures SAY output
        struct TestActions {
            output: Vec<String>,
        }
        impl ScriptActions for TestActions {
            fn say(&mut self, message: &str) {
                self.output.push(message.to_string());
            }
            fn chat(&mut self, _message: &str) {}
            fn local_msg(&mut self, _message: &str) {}
            fn room_msg(&mut self, _message: &str) {}
            fn private_msg(&mut self, _user_id: i32, _message: &str) {}
            fn goto_room(&mut self, _room_id: i16) {}
            fn lock_door(&mut self, _door_id: i32) {}
            fn unlock_door(&mut self, _door_id: i32) {}
            fn set_face(&mut self, _face_id: i16) {}
            fn set_color(&mut self, _color: i16) {}
            fn set_props(&mut self, _props: Vec<AssetSpec>) {}
            fn set_pos(&mut self, _x: i16, _y: i16) {}
            fn move_user(&mut self, _dx: i16, _dy: i16) {}
            fn goto_url(&mut self, _url: &str) {}
            fn goto_url_frame(&mut self, _url: &str, _frame: &str) {}
            fn global_msg(&mut self, _message: &str) {}
            fn status_msg(&mut self, _message: &str) {}
            fn superuser_msg(&mut self, _message: &str) {}
            fn log_msg(&mut self, _message: &str) {}
            fn set_spot_state(&mut self, _spot_id: i32, _state: i32) {}
            fn add_loose_prop(&mut self, _prop_id: i32, _x: i16, _y: i16) {}
            fn clear_loose_props(&mut self) {}
            fn play_sound(&mut self, _sound_id: i32) {}
            fn play_midi(&mut self, _midi_id: i32) {}
            fn stop_midi(&mut self) {}
            fn beep(&mut self) {}
            fn launch_app(&mut self, _url: &str) {}
        }

        // Test a script with variables and arithmetic
        let source = r#"
            ON SELECT {
                counter
                1 +
                counter =
                counter ITOA " clicks" & SAY
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();

        let mut actions = TestActions {
            output: Vec::new(),
        };
        let mut vm = Vm::new();
        vm.set_variable("counter".to_string(), Value::Integer(0));

        // Click once
        {
            let mut context = ScriptContext::new(SecurityLevel::Server, &mut actions);
            context.event_type = EventType::Select;
            vm.execute_handler(&script, EventType::Select, &mut context)
                .unwrap();
        }
        assert_eq!(actions.output, vec!["1 clicks"]);
        assert_eq!(vm.get_variable("counter"), Some(&Value::Integer(1)));

        // Click again
        actions.output.clear();
        {
            let mut context = ScriptContext::new(SecurityLevel::Server, &mut actions);
            context.event_type = EventType::Select;
            vm.execute_handler(&script, EventType::Select, &mut context)
                .unwrap();
        }
        assert_eq!(actions.output, vec!["2 clicks"]);
        assert_eq!(vm.get_variable("counter"), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_vm_integration_security() {
        use crate::iptscrae::{EventType, Lexer, Parser, ScriptContext, SecurityLevel};

        // Test that cyborg scripts can't lock doors
        let source = r#"
            ON SELECT {
                1 LOCK
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();

        let mut actions = ();
        let mut context = ScriptContext::new(SecurityLevel::Cyborg, &mut actions);
        context.event_type = EventType::Select;

        let mut vm = Vm::new();
        let result = vm.execute_handler(&script, EventType::Select, &mut context);

        assert!(matches!(result, Err(VmError::TypeError { .. })));
    }

    #[test]
    fn test_vm_props_functions() {
        use crate::iptscrae::{EventType, Lexer, Parser, ScriptActions, ScriptContext, SecurityLevel};
        use crate::AssetSpec;

        struct TestActions {
            color: i16,
            props: Vec<AssetSpec>,
        }

        impl ScriptActions for TestActions {
            fn say(&mut self, _message: &str) {}
            fn chat(&mut self, _message: &str) {}
            fn local_msg(&mut self, _message: &str) {}
            fn room_msg(&mut self, _message: &str) {}
            fn private_msg(&mut self, _user_id: i32, _message: &str) {}
            fn goto_room(&mut self, _room_id: i16) {}
            fn lock_door(&mut self, _door_id: i32) {}
            fn unlock_door(&mut self, _door_id: i32) {}
            fn set_face(&mut self, _face_id: i16) {}
            fn set_color(&mut self, color: i16) {
                self.color = color;
            }
            fn set_props(&mut self, props: Vec<AssetSpec>) {
                self.props = props;
            }
            fn set_pos(&mut self, _x: i16, _y: i16) {}
            fn move_user(&mut self, _dx: i16, _dy: i16) {}
            fn goto_url(&mut self, _url: &str) {}
            fn goto_url_frame(&mut self, _url: &str, _frame: &str) {}
            fn global_msg(&mut self, _message: &str) {}
            fn status_msg(&mut self, _message: &str) {}
            fn superuser_msg(&mut self, _message: &str) {}
            fn log_msg(&mut self, _message: &str) {}
            fn set_spot_state(&mut self, _spot_id: i32, _state: i32) {}
            fn add_loose_prop(&mut self, _prop_id: i32, _x: i16, _y: i16) {}
            fn clear_loose_props(&mut self) {}
            fn play_sound(&mut self, _sound_id: i32) {}
            fn play_midi(&mut self, _midi_id: i32) {}
            fn stop_midi(&mut self) {}
            fn beep(&mut self) {}
            fn launch_app(&mut self, _url: &str) {}
        }

        // Test SETCOLOR
        let source = r#"
            ON SELECT {
                5 SETCOLOR
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();

        let mut actions = TestActions {
            color: 0,
            props: Vec::new(),
        };
        {
            let mut context = ScriptContext::new(SecurityLevel::Server, &mut actions);
            context.event_type = EventType::Select;

            let mut vm = Vm::new();
            vm.execute_handler(&script, EventType::Select, &mut context)
                .unwrap();
        }

        assert_eq!(actions.color, 5);

        // Test GETPROPS
        let source = r#"
            ON SELECT {
                GETPROPS
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();

        let mut actions = TestActions {
            color: 0,
            props: Vec::new(),
        };
        {
            let mut context = ScriptContext::new(SecurityLevel::Server, &mut actions);
            context.event_type = EventType::Select;
            context.user_props = vec![
                AssetSpec { id: 100, crc: 12345 },
                AssetSpec { id: 200, crc: 67890 },
            ];

            let mut vm = Vm::new();
            vm.execute_handler(&script, EventType::Select, &mut context)
                .unwrap();

            // Stack should have: num_props, crc1, id1, crc2, id2
            assert_eq!(vm.stack().len(), 5);
            assert_eq!(vm.stack()[0], Value::Integer(2)); // num_props
            assert_eq!(vm.stack()[1], Value::Integer(12345)); // crc1
            assert_eq!(vm.stack()[2], Value::Integer(100)); // id1
            assert_eq!(vm.stack()[3], Value::Integer(67890)); // crc2
            assert_eq!(vm.stack()[4], Value::Integer(200)); // id2
        }

        // Test SETPROPS
        let source = r#"
            ON SELECT {
                11111 300 22222 400 2 SETPROPS
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let script = parser.parse().unwrap();

        let mut actions = TestActions {
            color: 0,
            props: Vec::new(),
        };
        {
            let mut context = ScriptContext::new(SecurityLevel::Server, &mut actions);
            context.event_type = EventType::Select;

            let mut vm = Vm::new();
            vm.execute_handler(&script, EventType::Select, &mut context)
                .unwrap();
        }

        assert_eq!(actions.props.len(), 2);
        assert_eq!(actions.props[0].id, 400);
        assert_eq!(actions.props[0].crc, 22222);
        assert_eq!(actions.props[1].id, 300);
        assert_eq!(actions.props[1].crc, 11111);
    }

    #[test]
    fn test_phase1_stack_operations() {
        let mut vm = Vm::new();

        // Test POP (alias for DROP)
        vm.push(Value::Integer(42));
        vm.push(Value::Integer(99));
        vm.execute_builtin_with_context("POP", None).unwrap();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack[0], Value::Integer(42));

        // Test STACKDEPTH
        vm.push(Value::Integer(1));
        vm.push(Value::Integer(2));
        // Stack now has: 42, 1, 2 (3 items)
        vm.execute_builtin_with_context("STACKDEPTH", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(3));

        // Test TOPTYPE
        vm.push(Value::Integer(123));
        vm.execute_builtin_with_context("TOPTYPE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1)); // 1 = integer

        vm.push(Value::String("test".to_string()));
        vm.execute_builtin_with_context("TOPTYPE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(2)); // 2 = string

        vm.push(Value::array(vec![Value::Integer(1), Value::Integer(2)]));
        vm.execute_builtin_with_context("TOPTYPE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(3)); // 3 = array

        // Test VARTYPE
        vm.set_variable("myint".to_string(), Value::Integer(42));
        vm.set_variable("mystr".to_string(), Value::String("hello".to_string()));
        vm.set_variable("myarr".to_string(), Value::array(vec![Value::Integer(1)]));

        vm.push(Value::String("myint".to_string()));
        vm.execute_builtin_with_context("VARTYPE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1)); // integer

        vm.push(Value::String("mystr".to_string()));
        vm.execute_builtin_with_context("VARTYPE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(2)); // string

        vm.push(Value::String("myarr".to_string()));
        vm.execute_builtin_with_context("VARTYPE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(3)); // array

        // Non-existent variable should return 0
        vm.push(Value::String("nonexistent".to_string()));
        vm.execute_builtin_with_context("VARTYPE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_phase1_string_operations() {
        let mut vm = Vm::new();

        // Test UPPERCASE
        vm.push(Value::String("hello world".to_string()));
        vm.execute_builtin_with_context("UPPERCASE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::String("HELLO WORLD".to_string()));

        // Test LOWERCASE
        vm.push(Value::String("HELLO WORLD".to_string()));
        vm.execute_builtin_with_context("LOWERCASE", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::String("hello world".to_string()));

        // Test SUBSTR - found
        vm.push(Value::String("hello world".to_string()));
        vm.push(Value::String("world".to_string()));
        vm.execute_builtin_with_context("SUBSTR", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));

        // Test SUBSTR - not found
        vm.push(Value::String("hello world".to_string()));
        vm.push(Value::String("xyz".to_string()));
        vm.execute_builtin_with_context("SUBSTR", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));

        // Test SUBSTRING
        vm.push(Value::String("hello world".to_string()));
        vm.push(Value::Integer(6)); // start
        vm.push(Value::Integer(5)); // length
        vm.execute_builtin_with_context("SUBSTRING", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::String("world".to_string()));

        // Test STRINDEX - found
        vm.push(Value::String("hello world".to_string()));
        vm.push(Value::String("world".to_string()));
        vm.execute_builtin_with_context("STRINDEX", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(6));

        // Test STRINDEX - not found
        vm.push(Value::String("hello world".to_string()));
        vm.push(Value::String("xyz".to_string()));
        vm.execute_builtin_with_context("STRINDEX", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(-1));
    }

    #[test]
    fn test_phase1_math_operations() {
        let mut vm = Vm::new();

        // Test RANDOM - should return 0..max-1
        vm.push(Value::Integer(100));
        vm.execute_builtin_with_context("RANDOM", None).unwrap();
        let result = vm.pop("test").unwrap();
        if let Value::Integer(n) = result {
            assert!(n >= 0 && n < 100);
        } else {
            panic!("RANDOM should return an integer");
        }

        // Test SQUAREROOT
        vm.push(Value::Integer(16));
        vm.execute_builtin_with_context("SQUAREROOT", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(4));

        vm.push(Value::Integer(100));
        vm.execute_builtin_with_context("SQUAREROOT", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(10));

        // Test SINE (returns sine * 1000)
        vm.push(Value::Integer(0));
        vm.execute_builtin_with_context("SINE", None).unwrap();
        let result = vm.pop("test").unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, 0);
        }

        vm.push(Value::Integer(90));
        vm.execute_builtin_with_context("SINE", None).unwrap();
        let result = vm.pop("test").unwrap();
        if let Value::Integer(n) = result {
            assert!((n - 1000).abs() < 10); // Should be close to 1000
        }

        // Test COSINE (returns cosine * 1000)
        vm.push(Value::Integer(0));
        vm.execute_builtin_with_context("COSINE", None).unwrap();
        let result = vm.pop("test").unwrap();
        if let Value::Integer(n) = result {
            assert!((n - 1000).abs() < 10); // Should be close to 1000
        }

        vm.push(Value::Integer(90));
        vm.execute_builtin_with_context("COSINE", None).unwrap();
        let result = vm.pop("test").unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, 0);
        }

        // Test TANGENT (returns tangent * 1000)
        vm.push(Value::Integer(0));
        vm.execute_builtin_with_context("TANGENT", None).unwrap();
        let result = vm.pop("test").unwrap();
        if let Value::Integer(n) = result {
            assert_eq!(n, 0);
        }

        vm.push(Value::Integer(45));
        vm.execute_builtin_with_context("TANGENT", None).unwrap();
        let result = vm.pop("test").unwrap();
        if let Value::Integer(n) = result {
            assert!((n - 1000).abs() < 10); // Should be close to 1000
        }
    }

    #[test]
    fn test_phase1_array_operations() {
        let mut vm = Vm::new();

        // Test ARRAY
        vm.push(Value::Integer(5));
        vm.execute_builtin_with_context("ARRAY", None).unwrap();
        let arr = vm.pop("test").unwrap();
        assert!(arr.is_array());
        if let Value::Array(ref a) = arr {
            assert_eq!(a.len(), 5);
            // All elements should be initialized to 0
            for elem in a {
                assert_eq!(*elem, Value::Integer(0));
            }
        }

        // Test PUT and GET
        // Create array [0, 0, 0]
        vm.push(Value::Integer(3));
        vm.execute_builtin_with_context("ARRAY", None).unwrap();
        let arr = vm.pop("test").unwrap();
        
        // PUT value 42 at index 1
        vm.push(arr.clone());
        vm.push(Value::Integer(1));
        vm.push(Value::Integer(42));
        vm.execute_builtin_with_context("PUT", None).unwrap();
        let arr = vm.pop("test").unwrap();

        // GET value at index 1
        vm.push(arr.clone());
        vm.push(Value::Integer(1));
        vm.execute_builtin_with_context("GET", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(42));

        // GET value at index 0 (should still be 0)
        vm.push(arr.clone());
        vm.push(Value::Integer(0));
        vm.execute_builtin_with_context("GET", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));

        // Test LENGTH on array
        vm.push(arr);
        vm.execute_builtin_with_context("LENGTH", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(3));

        // Test LENGTH on string
        vm.push(Value::String("hello".to_string()));
        vm.execute_builtin_with_context("LENGTH", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(5));

        // Test LENGTH on integer (should return 0)
        vm.push(Value::Integer(42));
        vm.execute_builtin_with_context("LENGTH", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_phase1_array_bounds_checking() {
        let mut vm = Vm::new();

        // Create array of size 3
        vm.push(Value::Integer(3));
        vm.execute_builtin_with_context("ARRAY", None).unwrap();
        let arr = vm.pop("test").unwrap();

        // Try to GET at negative index - should error
        vm.push(arr.clone());
        vm.push(Value::Integer(-1));
        let result = vm.execute_builtin_with_context("GET", None);
        assert!(matches!(result, Err(VmError::TypeError { .. })));

        // Try to GET at index >= length - should error
        vm.push(arr.clone());
        vm.push(Value::Integer(3));
        let result = vm.execute_builtin_with_context("GET", None);
        assert!(matches!(result, Err(VmError::TypeError { .. })));

        // Try to PUT at negative index - should error
        vm.push(arr.clone());
        vm.push(Value::Integer(-1));
        vm.push(Value::Integer(42));
        let result = vm.execute_builtin_with_context("PUT", None);
        assert!(matches!(result, Err(VmError::TypeError { .. })));

        // Try to PUT at index >= length - should error
        vm.push(arr.clone());
        vm.push(Value::Integer(3));
        vm.push(Value::Integer(42));
        let result = vm.execute_builtin_with_context("PUT", None);
        assert!(matches!(result, Err(VmError::TypeError { .. })));
    }
}
