//! User builtin functions for Palace.

use crate::iptscrae::context::{ScriptContext, SecurityLevel};
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute user builtin functions.
pub fn execute_user_builtin(
    vm: &mut Vm,
    name: &str,
    context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    match name {
        "USERNAME" => {
            vm.push_from_context_or(
                context.as_deref(),
                |ctx| Value::String(ctx.user_name.clone()),
                || Value::String("Guest".to_string()),
            );
            Ok(())
        }
        "WHOME" => {
            vm.push_from_context_or(
                context.as_deref(),
                |ctx| Value::Integer(ctx.user_id),
                || Value::Integer(0),
            );
            Ok(())
        }
        "WHONAME" => {
            let user_id = vm.pop("WHONAME")?.to_integer();
            if let Some(ctx) = context {
                // Look up username by ID (would need context support)
                // For now, just return user's own name if ID matches
                if user_id == ctx.user_id {
                    vm.push(Value::String(ctx.user_name.clone()));
                } else {
                    vm.push(Value::String(format!("User{}", user_id)));
                }
            } else {
                vm.push(Value::String("Guest".to_string()));
            }
            Ok(())
        }
        "SETFACE" => {
            let face_id = vm.pop("SETFACE")?.to_integer() as i16;
            vm.with_context_action(context, |ctx| ctx.actions.set_face(face_id));
            Ok(())
        }
        "SETCOLOR" => {
            let color = vm.pop("SETCOLOR")?.to_integer() as i16;
            vm.with_context_action(context, |ctx| ctx.actions.set_color(color));
            Ok(())
        }
        "WHOPOS" => {
            let user_id = vm.pop("WHOPOS")?.to_integer();
            // For now, return current user's position if ID matches
            if let Some(ctx) = context {
                if user_id == ctx.user_id {
                    vm.push(Value::Integer(ctx.user_pos_x as i32));
                    vm.push(Value::Integer(ctx.user_pos_y as i32));
                } else {
                    // Would need to look up other user's position
                    vm.push(Value::Integer(0));
                    vm.push(Value::Integer(0));
                }
            } else {
                vm.push(Value::Integer(0));
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "USERID" => {
            // USERID is an alias for WHOME
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_id));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "WHOTARGET" => {
            // Get targeted user ID - would need event data
            if let Some(ctx) = context {
                if let Some(Value::Integer(user_id)) = ctx.event_data.get("target_user_id") {
                    vm.push(Value::Integer(*user_id));
                } else {
                    vm.push(Value::Integer(0));
                }
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "ISGOD" => {
            // Check if current user has god/wizard privileges
            if let Some(ctx) = context {
                let is_god = matches!(ctx.security_level, SecurityLevel::Admin);
                vm.push(Value::Integer(if is_god { 1 } else { 0 }));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "ISWIZARD" => {
            // Alias for ISGOD
            if let Some(ctx) = context {
                let is_wizard = matches!(ctx.security_level, SecurityLevel::Admin);
                vm.push(Value::Integer(if is_wizard { 1 } else { 0 }));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "ISGUEST" => {
            // Check if user is a guest (would need user flags)
            // For now, return 0 (not a guest)
            vm.push(Value::Integer(0));
            Ok(())
        }
        "MOUSEPOS" => {
            // Get mouse position - would need UI state
            // Push X and Y coordinates
            vm.push(Value::Integer(0));
            vm.push(Value::Integer(0));
            Ok(())
        }
        "ME" => {
            // Return current user ID
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_id));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
