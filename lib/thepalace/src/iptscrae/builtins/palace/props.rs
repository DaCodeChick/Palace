//! Props builtin functions for Palace.

use crate::iptscrae::context::ScriptContext;
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute props builtin functions.
pub fn execute_props_builtin(
    vm: &mut Vm,
    name: &str,
    context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    match name {
        "GETPROPS" => {
            if let Some(ctx) = context {
                // Push number of props first, then each prop as crc:id
                let num_props = ctx.user_props.len() as i32;
                vm.push(Value::Integer(num_props));

                // Push each prop's CRC and ID
                for prop in &ctx.user_props {
                    vm.push(Value::Integer(prop.crc as i32));
                    vm.push(Value::Integer(prop.id));
                }
            } else {
                vm.push(Value::Integer(0)); // No props
            }
            Ok(())
        }
        "SETPROPS" => {
            // Pop number of props
            let num_props = vm.pop("SETPROPS num_props")?.to_integer();
            if num_props < 0 {
                return Err(VmError::TypeError {
                    message: "SETPROPS num_props must be non-negative".to_string(),
                });
            }

            let mut props = Vec::new();
            for _ in 0..num_props {
                let id = vm.pop("SETPROPS prop id")?.to_integer();
                let crc = vm.pop("SETPROPS prop crc")?.to_integer();
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
        "NAKED" => {
            // Remove all props
            if let Some(ctx) = context {
                ctx.actions.set_props(Vec::new());
            }
            Ok(())
        }
        "DONPROP" => {
            // Add a prop: crc id -> add prop to user
            let id = vm.pop("DONPROP id")?.to_integer();
            let crc = vm.pop("DONPROP crc")?.to_integer();
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
            let id = vm.pop("DOFFPROP id")?.to_integer();
            if let Some(ctx) = context {
                let props: Vec<_> = ctx
                    .user_props
                    .iter()
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
            let id = vm.pop("REMOVEPROP id")?.to_integer();
            if let Some(ctx) = context {
                let props: Vec<_> = ctx
                    .user_props
                    .iter()
                    .filter(|p| p.id != id)
                    .cloned()
                    .collect();
                ctx.actions.set_props(props);
            }
            Ok(())
        }
        "USERPROP" => {
            // Get prop at index: index -> crc id
            let index = vm.pop("USERPROP")?.to_integer();
            if let Some(ctx) = context {
                if index >= 0 && (index as usize) < ctx.user_props.len() {
                    let prop = &ctx.user_props[index as usize];
                    vm.push(Value::Integer(prop.crc as i32));
                    vm.push(Value::Integer(prop.id));
                } else {
                    vm.push(Value::Integer(0));
                    vm.push(Value::Integer(0));
                }
            } else {
                vm.push(Value::Integer(0));
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "NBRUSERPROPS" => {
            // Get number of props user is wearing
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_props.len() as i32));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "TOPPROP" => {
            // Get the top (last) prop: -> crc id
            if let Some(ctx) = context {
                if let Some(prop) = ctx.user_props.last() {
                    vm.push(Value::Integer(prop.crc as i32));
                    vm.push(Value::Integer(prop.id));
                } else {
                    vm.push(Value::Integer(0));
                    vm.push(Value::Integer(0));
                }
            } else {
                vm.push(Value::Integer(0));
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "HASPROP" => {
            // Check if user has a specific prop: id -> boolean
            let id = vm.pop("HASPROP")?.to_integer();
            if let Some(ctx) = context {
                let has_prop = ctx.user_props.iter().any(|p| p.id == id);
                vm.push(Value::Integer(if has_prop { 1 } else { 0 }));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "ADDLOOSEPROP" => {
            // Add a loose prop to the room
            let y = vm.pop("ADDLOOSEPROP y")?.to_integer();
            let x = vm.pop("ADDLOOSEPROP x")?.to_integer();
            let prop_id = vm.pop("ADDLOOSEPROP prop_id")?.to_integer();
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
            let _show = vm.pop("SHOWLOOSEPROPS")?.to_integer();
            // For now, just consume the parameter
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
