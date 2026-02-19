//! System builtin functions for Palace.

use crate::iptscrae::context::ScriptContext;
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute system builtin functions.
pub fn execute_system_builtin(
    vm: &mut Vm,
    name: &str,
    context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    match name {
        "MACRO" => {
            // Execute a macro (prop script) - stub for now
            let _macro_id = vm.pop("MACRO")?.to_integer();
            // Would need to look up and execute the macro script
            Ok(())
        }
        "SERVERNAME" => {
            if let Some(ctx) = context {
                vm.push(Value::String(ctx.server_name.clone()));
            } else {
                vm.push(Value::String("localhost".to_string()));
            }
            Ok(())
        }
        "CLIENTTYPE" => {
            // Return client type identifier
            vm.push(Value::String("Palace".to_string()));
            Ok(())
        }
        "IPTVERSION" => {
            // Return Iptscrae version
            vm.push(Value::Integer(1));
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
            vm.push(Value::String(format!("{}", now)));
            Ok(())
        }
        "TICKS" => {
            // Return ticks (milliseconds since start)
            use std::time::SystemTime;
            let ticks = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i32;
            vm.push(Value::Integer(ticks));
            Ok(())
        }
        "DELAY" => {
            // Delay execution - not implemented (would need async/timer support)
            let _milliseconds = vm.pop("DELAY")?.to_integer();
            Ok(())
        }
        "GLOBAL" => {
            // Access global variable - would need global variable storage
            let var_name = vm.pop("GLOBAL")?.to_string();
            // For now, treat as regular variable
            if let Some(value) = vm.get_variable(&var_name) {
                vm.push(value.clone());
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "ID" => {
            // Alias for ME
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_id));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "SOUND" => {
            let sound_id = vm.pop("SOUND")?.to_integer();
            if let Some(ctx) = context {
                ctx.actions.play_sound(sound_id);
            }
            Ok(())
        }
        "MIDIPLAY" => {
            let midi_id = vm.pop("MIDIPLAY")?.to_integer();
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
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
