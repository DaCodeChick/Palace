//! Virtual Machine for Iptscrae script execution.
//!
//! The VM is a stack-based interpreter that executes Iptscrae AST nodes.
//! It maintains a value stack and variable storage, executing operations
//! by pushing/popping values from the stack.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::iptscrae::ast::{BinOp, Block, Expr, Script, Statement, UnaryOp};
use crate::iptscrae::context::ScriptContext;
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
    /// Maximum number of instructions to execute
    pub max_instructions: Option<usize>,
    /// Maximum execution time
    pub max_duration: Option<Duration>,
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

    /// Execute a built-in function with optional context
    fn execute_builtin_with_context(
        &mut self,
        name: &str,
        context: Option<&mut ScriptContext>,
    ) -> Result<(), VmError> {
        match name.to_uppercase().as_str() {
            // Stack operations
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

            // String operations
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
            "TOUPPER" => {
                let value = self.pop("TOUPPER")?;
                self.push(Value::String(value.to_string().to_uppercase()));
                Ok(())
            }
            "TOLOWER" => {
                let value = self.pop("TOLOWER")?;
                self.push(Value::String(value.to_string().to_lowercase()));
                Ok(())
            }

            // Palace operations - require context
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
            "ROOMNAME" => {
                if let Some(ctx) = context {
                    self.push(Value::String(ctx.room_name.clone()));
                } else {
                    self.push(Value::String("".to_string()));
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

            _ => Err(VmError::UndefinedFunction {
                name: name.to_string(),
            }),
        }
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
        let limits = ExecutionLimits {
            max_instructions: Some(10),
            max_duration: None,
        };
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

        // Test TOUPPER
        vm.push(Value::String("hello".to_string()));
        vm.execute_builtin_with_context("TOUPPER", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::String("HELLO".to_string()));

        // Test TOLOWER
        vm.push(Value::String("WORLD".to_string()));
        vm.execute_builtin_with_context("TOLOWER", None).unwrap();
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
            fn set_props(&mut self, _props: Vec<AssetSpec>) {}
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
            fn set_props(&mut self, _props: Vec<AssetSpec>) {}
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
}
