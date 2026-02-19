//! Array builtin functions for Iptscrae VM.

use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute array builtin functions.
pub fn execute_array_builtin(vm: &mut Vm, name: &str) -> Result<(), VmError> {
    match name {
        "ARRAY" => {
            // ARRAY takes size from stack, creates array of that size
            let size = vm.pop("ARRAY")?.to_integer();
            if size < 0 {
                return Err(VmError::TypeError {
                    message: "ARRAY size must be non-negative".to_string(),
                });
            }
            let arr = vec![Value::Integer(0); size as usize];
            vm.push(Value::Array(arr));
            Ok(())
        }
        "GET" => {
            // GET: array index -> value
            let index = vm.pop("GET index")?.to_integer();
            let array = vm.pop("GET array")?;

            if let Some(arr) = array.as_array() {
                if index < 0 || index >= arr.len() as i32 {
                    return Err(VmError::TypeError {
                        message: format!("Array index {} out of bounds", index),
                    });
                }
                vm.push(arr[index as usize].clone());
            } else {
                return Err(VmError::TypeError {
                    message: "GET requires an array".to_string(),
                });
            }
            Ok(())
        }
        "PUT" => {
            // PUT: array index value -> array (modified)
            let value = vm.pop("PUT value")?;
            let index = vm.pop("PUT index")?.to_integer();
            let mut array = vm.pop("PUT array")?;

            if let Some(arr) = array.as_array_mut() {
                if index < 0 || index >= arr.len() as i32 {
                    return Err(VmError::TypeError {
                        message: format!("Array index {} out of bounds", index),
                    });
                }
                arr[index as usize] = value;
                vm.push(array);
            } else {
                return Err(VmError::TypeError {
                    message: "PUT requires an array".to_string(),
                });
            }
            Ok(())
        }
        "LENGTH" => {
            // LENGTH: array -> length (also works on strings)
            let value = vm.pop("LENGTH")?;
            let length = match value {
                Value::Array(ref arr) => arr.len() as i32,
                Value::String(ref s) => s.len() as i32,
                Value::Integer(_) => 0,
            };
            vm.push(Value::Integer(length));
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
