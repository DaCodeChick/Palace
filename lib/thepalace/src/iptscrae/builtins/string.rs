//! String manipulation builtin functions for Iptscrae VM.

use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute string manipulation builtin functions.
pub fn execute_string_builtin(vm: &mut Vm, name: &str) -> Result<(), VmError> {
    match name {
        "ITOA" => {
            let value = vm.pop("ITOA")?;
            vm.push(Value::String(value.to_integer().to_string()));
            Ok(())
        }
        "ATOI" => {
            let value = vm.pop("ATOI")?;
            vm.push(Value::Integer(value.to_integer()));
            Ok(())
        }
        "STRLEN" => {
            let value = vm.pop("STRLEN")?;
            vm.push(Value::Integer(value.to_string().len() as i32));
            Ok(())
        }
        "UPPERCASE" => {
            let value = vm.pop("UPPERCASE")?;
            vm.push(Value::String(value.to_string().to_uppercase()));
            Ok(())
        }
        "LOWERCASE" => {
            let value = vm.pop("LOWERCASE")?;
            vm.push(Value::String(value.to_string().to_lowercase()));
            Ok(())
        }
        "SUBSTR" => {
            // Search for substring - pushes 1 if found, 0 if not
            let needle = vm.pop("SUBSTR needle")?.to_string();
            let haystack = vm.pop("SUBSTR haystack")?.to_string();
            let found = if haystack.contains(&needle) { 1 } else { 0 };
            vm.push(Value::Integer(found));
            Ok(())
        }
        "SUBSTRING" => {
            // Extract substring: string start length -> substring
            let length = vm.pop("SUBSTRING length")?.to_integer();
            let start = vm.pop("SUBSTRING start")?.to_integer();
            let string = vm.pop("SUBSTRING string")?.to_string();

            if start < 0 || length < 0 {
                vm.push(Value::String(String::new()));
                return Ok(());
            }

            let start_idx = start as usize;
            let length_usize = length as usize;

            let result = string
                .chars()
                .skip(start_idx)
                .take(length_usize)
                .collect::<String>();

            vm.push(Value::String(result));
            Ok(())
        }
        "STRINDEX" => {
            // Find index of substring: haystack needle -> index (or -1 if not found)
            let needle = vm.pop("STRINDEX needle")?.to_string();
            let haystack = vm.pop("STRINDEX haystack")?.to_string();

            let index = haystack.find(&needle).map(|i| i as i32).unwrap_or(-1);

            vm.push(Value::Integer(index));
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
