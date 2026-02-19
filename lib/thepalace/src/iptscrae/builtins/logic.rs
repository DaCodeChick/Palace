//! Logic builtin functions for Iptscrae VM.

use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute logic builtin functions.
pub fn execute_logic_builtin(vm: &mut Vm, name: &str) -> Result<(), VmError> {
    match name {
        "AND" => {
            // AND: a b -> (a AND b)
            let right = vm.pop("AND right")?.to_bool();
            let left = vm.pop("AND left")?.to_bool();
            vm.push(Value::Integer(if left && right { 1 } else { 0 }));
            Ok(())
        }
        "OR" => {
            // OR: a b -> (a OR b)
            let right = vm.pop("OR right")?.to_bool();
            let left = vm.pop("OR left")?.to_bool();
            vm.push(Value::Integer(if left || right { 1 } else { 0 }));
            Ok(())
        }
        "XOR" => {
            // XOR: a b -> (a XOR b)
            let right = vm.pop("XOR right")?.to_bool();
            let left = vm.pop("XOR left")?.to_bool();
            vm.push(Value::Integer(if left != right { 1 } else { 0 }));
            Ok(())
        }
        "NOT" => {
            // NOT: a -> (NOT a)
            let value = vm.pop("NOT")?.to_bool();
            vm.push(Value::Integer(if value { 0 } else { 1 }));
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
