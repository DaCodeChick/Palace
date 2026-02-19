//! Math builtin functions for Iptscrae VM.

use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute math builtin functions.
pub fn execute_math_builtin(vm: &mut Vm, name: &str) -> Result<(), VmError> {
    // Macro for trigonometric functions (SINE, COSINE, TANGENT)
    macro_rules! trig_builtin {
        ($name:expr, $func:ident) => {{
            let degrees = vm.pop($name)?.to_integer();
            let radians = (degrees as f64).to_radians();
            let result = (radians.$func() * 1000.0) as i32;
            vm.push(Value::Integer(result));
            Ok(())
        }};
    }

    match name {
        "RANDOM" => {
            // RANDOM takes max value from stack, returns random 0..max
            let max = vm.pop("RANDOM")?.to_integer();
            if max <= 0 {
                vm.push(Value::Integer(0));
            } else {
                // Simple pseudo-random using instruction count as seed
                let random_val = (vm.instruction_count() as i32 * 1103515245 + 12345) % max;
                vm.push(Value::Integer(random_val.abs()));
            }
            Ok(())
        }
        "SQUAREROOT" => {
            let value = vm.pop("SQUAREROOT")?.to_integer();
            let result = if value >= 0 {
                (value as f64).sqrt() as i32
            } else {
                0
            };
            vm.push(Value::Integer(result));
            Ok(())
        }
        "SINE" => trig_builtin!("SINE", sin),
        "COSINE" => trig_builtin!("COSINE", cos),
        "TANGENT" => trig_builtin!("TANGENT", tan),
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
