//! Messaging builtin functions for Palace.

use crate::iptscrae::context::{ScriptContext, SecurityLevel};
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute messaging builtin functions.
pub fn execute_messaging_builtin(
    vm: &mut Vm,
    name: &str,
    context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    match name {
        "SAY" => {
            let message = vm.pop("SAY")?;
            if let Some(ctx) = context {
                ctx.actions.say(&message.to_string());
            } else {
                // Fallback for tests
                vm.push_output(message.to_string());
            }
            Ok(())
        }
        "CHAT" => {
            let message = vm.pop("CHAT")?;
            if let Some(ctx) = context {
                ctx.actions.chat(&message.to_string());
            } else {
                // Fallback for tests
                vm.push_output(message.to_string());
            }
            Ok(())
        }
        "LOCALMSG" => {
            let message = vm.pop("LOCALMSG")?;
            if let Some(ctx) = context {
                ctx.actions.local_msg(&message.to_string());
            }
            Ok(())
        }
        "ROOMMSG" => {
            let message = vm.pop("ROOMMSG")?;
            if let Some(ctx) = context {
                ctx.actions.room_msg(&message.to_string());
            }
            Ok(())
        }
        "PRIVATEMSG" => {
            let message = vm.pop("PRIVATEMSG")?;
            let user_id = vm.pop("PRIVATEMSG user_id")?.to_integer();
            if let Some(ctx) = context {
                ctx.actions.private_msg(user_id, &message.to_string());
            }
            Ok(())
        }
        "WHOCHAT" => {
            // Get user ID from last chat message - would need event data
            if let Some(ctx) = context {
                if let Some(Value::Integer(user_id)) = ctx.event_data.get("chat_user_id") {
                    vm.push(Value::Integer(*user_id));
                } else {
                    vm.push(Value::Integer(ctx.user_id));
                }
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "SAYAT" => {
            let y = vm.pop("SAYAT y")?.to_integer();
            let x = vm.pop("SAYAT x")?.to_integer();
            let message = vm.pop("SAYAT message")?.to_string();
            if let Some(ctx) = context {
                // SAYAT displays text at a specific position
                // Format as special message with coordinates
                let msg = format!("@{},{}: {}", x, y, message);
                ctx.actions.say(&msg);
            }
            Ok(())
        }
        "GLOBALMSG" => {
            let message = vm.pop("GLOBALMSG")?.to_string();
            if let Some(ctx) = context {
                ctx.actions.global_msg(&message);
            }
            Ok(())
        }
        "STATUSMSG" => {
            let message = vm.pop("STATUSMSG")?.to_string();
            if let Some(ctx) = context {
                ctx.actions.status_msg(&message);
            }
            Ok(())
        }
        "SUSRMSG" => {
            let message = vm.pop("SUSRMSG")?.to_string();
            if let Some(ctx) = context {
                if !matches!(ctx.security_level, SecurityLevel::Admin) {
                    return Err(VmError::TypeError {
                        message: "SUSRMSG requires admin privileges".to_string(),
                    });
                }
                ctx.actions.superuser_msg(&message);
            }
            Ok(())
        }
        "LOGMSG" => {
            let message = vm.pop("LOGMSG")?.to_string();
            if let Some(ctx) = context {
                ctx.actions.log_msg(&message);
            }
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
