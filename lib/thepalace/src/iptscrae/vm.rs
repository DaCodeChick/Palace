//! Virtual Machine for Iptscrae script execution.
//!
//! The VM is a stack-based interpreter that executes Iptscrae AST nodes.
//! It maintains a value stack and variable storage, executing operations
//! by pushing/popping values from the stack.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::iptscrae::ast::{BinOp, Block, Expr, Script, Statement, UnaryOp};
use crate::iptscrae::builtins;
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
    /// Security violation - function not allowed at current security level
    SecurityViolation { function: String },
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
            VmError::SecurityViolation { function } => {
                write!(f, "Security violation: {} not allowed at this security level", function)
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

    /// Execute a built-in function with optional context
    fn execute_builtin_with_context(
        &mut self,
        name: &str,
        context: Option<&mut ScriptContext>,
    ) -> Result<(), VmError> {
        let name_upper = name.to_uppercase();
        let name_str = name_upper.as_str();

        // Try stack operations first (most common)
        match builtins::execute_stack_builtin(self, name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try string operations
        match builtins::execute_string_builtin(self, name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try math operations
        match builtins::execute_math_builtin(self, name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try logic operations
        match builtins::execute_logic_builtin(self, name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try array operations
        match builtins::execute_array_builtin(self, name_str) {
            Ok(()) => return Ok(()),
            Err(VmError::UndefinedFunction { .. }) => {}
            Err(e) => return Err(e),
        }

        // Try Palace operations
        builtins::execute_palace_builtin(self, name_str, context)
    }

    /// Push a value onto the stack
    pub(crate) fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop a value from the stack
    pub(crate) fn pop(&mut self, operation: &str) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| VmError::StackUnderflow {
            operation: operation.to_string(),
        })
    }

    /// Peek at top value without removing it
    pub(crate) fn peek(&self, operation: &str) -> Result<Value, VmError> {
        self.stack
            .last()
            .cloned()
            .ok_or_else(|| VmError::StackUnderflow {
                operation: operation.to_string(),
            })
    }

    /// Get stack length (for builtin modules)
    pub(crate) fn stack_len(&self) -> usize {
        self.stack.len()
    }

    /// Get stack element at index (for builtin modules)
    pub(crate) fn stack_get(&self, index: usize) -> &Value {
        &self.stack[index]
    }

    /// Get instruction count (for builtin modules like RANDOM)
    pub(crate) fn instruction_count(&self) -> usize {
        self.instruction_count
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

    /// Push to output buffer (for builtin modules)
    pub(crate) fn push_output(&mut self, message: String) {
        self.output.push(message);
    }

    /// Clear output buffer
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Helper: Push a value from context or a default value
    pub(crate) fn push_from_context_or<F, D>(&mut self, context: Option<&ScriptContext>, f: F, default: D)
    where
        F: FnOnce(&ScriptContext) -> Value,
        D: FnOnce() -> Value,
    {
        if let Some(ctx) = context {
            self.push(f(ctx));
        } else {
            self.push(default());
        }
    }

    /// Helper: Execute an action with context if available
    pub(crate) fn with_context_action<F>(&self, context: Option<&mut ScriptContext>, f: F)
    where
        F: FnOnce(&mut ScriptContext),
    {
        if let Some(ctx) = context {
            f(ctx);
        }
    }

    /// Helper: Check if function is allowed at current security level
    pub(crate) fn require_permission(
        &self,
        context: Option<&ScriptContext>,
        function: &str,
    ) -> Result<(), VmError> {
        match context {
            Some(ctx) if ctx.is_function_allowed(function) => Ok(()),
            Some(_) => Err(VmError::SecurityViolation {
                function: function.to_string(),
            }),
            None => Ok(()), // Allow in test mode without context
        }
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

    /// Helper: Parse Iptscrae source code into a Script
    #[allow(dead_code)]
    fn parse_script(source: &str) -> Result<Script, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {:?}", e))?;
        let mut parser = Parser::new(tokens);
        parser.parse().map_err(|e| format!("Parser error: {:?}", e))
    }

    /// Helper: Execute a builtin function with setup
    fn test_builtin<F>(builtin: &str, setup: F) -> Vm
    where
        F: FnOnce(&mut Vm),
    {
        let mut vm = Vm::new();
        setup(&mut vm);
        vm.execute_builtin_with_context(builtin, None).unwrap();
        vm
    }

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
        // Test PICK
        let vm = test_builtin("PICK", |vm| {
            vm.push(Value::Integer(1));
            vm.push(Value::Integer(2));
            vm.push(Value::Integer(3));
            vm.push(Value::Integer(1)); // Pick index 1 (should get value 2)
        });
        assert_eq!(vm.stack().last(), Some(&Value::Integer(2)));

        // Test STRLEN
        let vm = test_builtin("STRLEN", |vm| {
            vm.push(Value::String("hello".to_string()));
        });
        assert_eq!(vm.stack().last(), Some(&Value::Integer(5)));

        // Test UPPERCASE
        let vm = test_builtin("UPPERCASE", |vm| {
            vm.push(Value::String("hello".to_string()));
        });
        assert_eq!(vm.stack().last(), Some(&Value::String("HELLO".to_string())));

        // Test LOWERCASE
        let vm = test_builtin("LOWERCASE", |vm| {
            vm.push(Value::String("WORLD".to_string()));
        });
        assert_eq!(vm.stack().last(), Some(&Value::String("world".to_string())));
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

        assert!(matches!(result, Err(VmError::SecurityViolation { .. })));
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

    #[test]
    fn test_logic_operations() {
        let mut vm = Vm::new();

        // Test AND
        vm.push(Value::Integer(1)); // true
        vm.push(Value::Integer(1)); // true
        vm.execute_builtin_with_context("AND", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));

        vm.push(Value::Integer(1)); // true
        vm.push(Value::Integer(0)); // false
        vm.execute_builtin_with_context("AND", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));

        // Test OR
        vm.push(Value::Integer(0)); // false
        vm.push(Value::Integer(1)); // true
        vm.execute_builtin_with_context("OR", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));

        vm.push(Value::Integer(0)); // false
        vm.push(Value::Integer(0)); // false
        vm.execute_builtin_with_context("OR", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));

        // Test XOR
        vm.push(Value::Integer(1)); // true
        vm.push(Value::Integer(0)); // false
        vm.execute_builtin_with_context("XOR", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));

        vm.push(Value::Integer(1)); // true
        vm.push(Value::Integer(1)); // true
        vm.execute_builtin_with_context("XOR", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));

        // Test NOT
        vm.push(Value::Integer(1)); // true
        vm.execute_builtin_with_context("NOT", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(0));

        vm.push(Value::Integer(0)); // false
        vm.execute_builtin_with_context("NOT", None).unwrap();
        assert_eq!(vm.pop("test").unwrap(), Value::Integer(1));
    }
}
