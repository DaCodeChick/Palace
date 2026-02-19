//! Stack manipulation builtin functions for Iptscrae VM.

use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute stack manipulation builtin functions.
pub fn execute_stack_builtin(vm: &mut Vm, name: &str) -> Result<(), VmError> {
    match name {
        "DUP" => {
            let value = vm.peek("DUP")?;
            vm.push(value);
            Ok(())
        }
        "DROP" => {
            vm.pop("DROP")?;
            Ok(())
        }
        "SWAP" => {
            let a = vm.pop("SWAP first")?;
            let b = vm.pop("SWAP second")?;
            vm.push(a);
            vm.push(b);
            Ok(())
        }
        "OVER" => {
            // Copy second value to top: a b -> a b a
            if vm.stack_len() < 2 {
                return Err(VmError::StackUnderflow {
                    operation: "OVER".to_string(),
                });
            }
            let value = vm.stack_get(vm.stack_len() - 2).clone();
            vm.push(value);
            Ok(())
        }
        "ROT" => {
            // Rotate top three: a b c -> b c a
            if vm.stack_len() < 3 {
                return Err(VmError::StackUnderflow {
                    operation: "ROT".to_string(),
                });
            }
            let c = vm.pop("ROT")?;
            let b = vm.pop("ROT")?;
            let a = vm.pop("ROT")?;
            vm.push(b);
            vm.push(c);
            vm.push(a);
            Ok(())
        }
        "PICK" => {
            // Copy nth value to top (0-indexed from top)
            let n = vm.pop("PICK")?.to_integer();
            if n < 0 {
                return Err(VmError::TypeError {
                    message: "PICK index must be non-negative".to_string(),
                });
            }
            let index = n as usize;
            if index >= vm.stack_len() {
                return Err(VmError::StackUnderflow {
                    operation: "PICK".to_string(),
                });
            }
            let value = vm.stack_get(vm.stack_len() - 1 - index).clone();
            vm.push(value);
            Ok(())
        }
        "POP" => {
            // Alias for DROP
            vm.pop("POP")?;
            Ok(())
        }
        "STACKDEPTH" => {
            vm.push(Value::Integer(vm.stack_len() as i32));
            Ok(())
        }
        "TOPTYPE" => {
            let value = vm.peek("TOPTYPE")?;
            let type_id = match value {
                Value::Integer(_) => 1,
                Value::String(_) => 2,
                Value::Array(_) => 3,
            };
            vm.push(Value::Integer(type_id));
            Ok(())
        }
        "VARTYPE" => {
            // Get type of variable - needs variable name from stack
            let var_name = vm.pop("VARTYPE")?.to_string();
            if let Some(value) = vm.get_variable(&var_name) {
                let type_id = match value {
                    Value::Integer(_) => 1,
                    Value::String(_) => 2,
                    Value::Array(_) => 3,
                };
                vm.push(Value::Integer(type_id));
            } else {
                vm.push(Value::Integer(0)); // Undefined = 0
            }
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
