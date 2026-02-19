//! Room builtin functions for Palace.

use crate::iptscrae::context::ScriptContext;
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute room builtin functions.
pub fn execute_room_builtin(
    vm: &mut Vm,
    name: &str,
    context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    match name {
        "ROOMNAME" => {
            vm.push_from_context_or(
                context.as_deref(),
                |ctx| Value::String(ctx.room_name.clone()),
                || Value::String(String::new()),
            );
            Ok(())
        }
        "ROOMID" => {
            vm.push_from_context_or(
                context.as_deref(),
                |ctx| Value::Integer(ctx.room_id as i32),
                || Value::Integer(0),
            );
            Ok(())
        }
        "LOCK" => {
            vm.require_permission(context.as_deref(), "LOCK")?;
            let door_id = vm.pop("LOCK")?.to_integer();
            vm.with_context_action(context, |ctx| ctx.actions.lock_door(door_id));
            Ok(())
        }
        "UNLOCK" => {
            vm.require_permission(context.as_deref(), "UNLOCK")?;
            let door_id = vm.pop("UNLOCK")?.to_integer();
            vm.with_context_action(context, |ctx| ctx.actions.unlock_door(door_id));
            Ok(())
        }
        "NBRROOMUSERS" => {
            // Number of users in current room - would need room state
            // For now, return 1 (just the current user)
            vm.push(Value::Integer(1));
            Ok(())
        }
        "ROOMUSER" => {
            // Get user ID by index in room - would need room state
            let index = vm.pop("ROOMUSER")?.to_integer();
            if let Some(ctx) = context {
                if index == 0 {
                    vm.push(Value::Integer(ctx.user_id));
                } else {
                    vm.push(Value::Integer(0));
                }
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "DOORIDX" => {
            // Get current door index - would need event data
            if let Some(ctx) = context {
                if let Some(Value::Integer(door_id)) = ctx.event_data.get("door_id") {
                    vm.push(Value::Integer(*door_id));
                } else {
                    vm.push(Value::Integer(-1));
                }
            } else {
                vm.push(Value::Integer(-1));
            }
            Ok(())
        }
        "NBRDOORS" => {
            // Get number of doors in room - would need room data
            // For now, return 0
            vm.push(Value::Integer(0));
            Ok(())
        }
        "ISLOCKED" => {
            // Check if door is locked - would need room state
            let _door_id = vm.pop("ISLOCKED")?.to_integer();
            // For now, return 0 (unlocked)
            vm.push(Value::Integer(0));
            Ok(())
        }
        "SPOTIDX" => {
            // Get current spot index - would need event data
            if let Some(ctx) = context {
                if let Some(Value::Integer(spot_id)) = ctx.event_data.get("spot_id") {
                    vm.push(Value::Integer(*spot_id));
                } else {
                    vm.push(Value::Integer(-1));
                }
            } else {
                vm.push(Value::Integer(-1));
            }
            Ok(())
        }
        "NBRSPOTS" => {
            // Get number of spots in room - would need room data
            // For now, return 0
            vm.push(Value::Integer(0));
            Ok(())
        }
        "SPOTNAME" => {
            // Get name of spot by ID - would need room data
            let _spot_id = vm.pop("SPOTNAME")?.to_integer();
            vm.push(Value::String(String::new()));
            Ok(())
        }
        "SPOTDEST" => {
            // Get destination for spot - would need room data
            let _spot_id = vm.pop("SPOTDEST")?.to_integer();
            // Returns room_id
            vm.push(Value::Integer(0));
            Ok(())
        }
        "INSPOT" => {
            // Check if user is in a specific spot - would need position/spot data
            let _spot_id = vm.pop("INSPOT")?.to_integer();
            // For now, return 0 (not in spot)
            vm.push(Value::Integer(0));
            Ok(())
        }
        "GETSPOTSTATE" => {
            // Get state of a spot
            let _spot_id = vm.pop("GETSPOTSTATE")?.to_integer();
            // For now, return 0
            vm.push(Value::Integer(0));
            Ok(())
        }
        "SETSPOTSTATE" => {
            // Set state of a spot (global)
            let state = vm.pop("SETSPOTSTATE state")?.to_integer();
            let spot_id = vm.pop("SETSPOTSTATE spot_id")?.to_integer();
            if let Some(ctx) = context {
                ctx.actions.set_spot_state(spot_id, state);
            }
            Ok(())
        }
        "SETSPOTSTATELOCAL" => {
            // Set state of a spot (local to user)
            let _state = vm.pop("SETSPOTSTATELOCAL state")?.to_integer();
            let _spot_id = vm.pop("SETSPOTSTATELOCAL spot_id")?.to_integer();
            // Would need local state storage
            Ok(())
        }
        "SETPICLOC" => {
            // Set picture location (for drawing) - would need graphics context
            let _y = vm.pop("SETPICLOC y")?.to_integer();
            let _x = vm.pop("SETPICLOC x")?.to_integer();
            // For now, just consume the parameters
            Ok(())
        }
        "DIMROOM" => {
            // Dim the room - UI effect
            let _brightness = vm.pop("DIMROOM")?.to_integer();
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
