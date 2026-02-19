//! Navigation builtin functions for Palace.

use crate::iptscrae::context::{ScriptContext, SecurityLevel};
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute navigation builtin functions.
pub fn execute_navigation_builtin(
    vm: &mut Vm,
    name: &str,
    context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    match name {
        "GOTOROOM" => {
            vm.require_permission(context.as_deref(), "GOTOROOM")?;
            let room_id = vm.pop("GOTOROOM")?.to_integer();
            vm.with_context_action(context, |ctx| ctx.actions.goto_room(room_id as i16));
            Ok(())
        }
        "MOVE" => {
            let dy = vm.pop("MOVE dy")?.to_integer();
            let dx = vm.pop("MOVE dx")?.to_integer();
            if let Some(ctx) = context {
                ctx.actions.move_user(dx as i16, dy as i16);
                ctx.user_pos_x += dx as i16;
                ctx.user_pos_y += dy as i16;
            }
            Ok(())
        }
        "GOTOURL" => {
            let url = vm.pop("GOTOURL")?.to_string();
            if let Some(ctx) = context {
                ctx.actions.goto_url(&url);
            }
            Ok(())
        }
        "GOTOURLFRAME" => {
            let frame = vm.pop("GOTOURLFRAME frame")?.to_string();
            let url = vm.pop("GOTOURLFRAME url")?.to_string();
            if let Some(ctx) = context {
                ctx.actions.goto_url_frame(&url, &frame);
            }
            Ok(())
        }
        "NETGOTO" => {
            // NETGOTO: server_name room_id -> navigate to room on another server
            let room_id = vm.pop("NETGOTO room_id")?.to_integer();
            let server = vm.pop("NETGOTO server")?.to_string();
            // Construct URL and delegate to GOTOURL
            let url = format!("palace://{}?room={}", server, room_id);
            if let Some(ctx) = context {
                ctx.actions.goto_url(&url);
            }
            Ok(())
        }
        "KILLUSER" => {
            // Disconnect a user - admin only
            let user_id = vm.pop("KILLUSER")?.to_integer();
            if let Some(ctx) = context {
                if !matches!(ctx.security_level, SecurityLevel::Admin) {
                    return Err(VmError::TypeError {
                        message: "KILLUSER requires admin privileges".to_string(),
                    });
                }
                // Would need server action to disconnect user
                // For now, just consume the parameter
                let _ = user_id;
            }
            Ok(())
        }
        "DEST" => {
            // Get destination room ID for a door - would need room data
            let _door_id = vm.pop("DEST")?.to_integer();
            // For now, return 0
            vm.push(Value::Integer(0));
            Ok(())
        }
        "SETLOC" => {
            // Set location (alias for SETPOS)
            let y = vm.pop("SETLOC y")?.to_integer();
            let x = vm.pop("SETLOC x")?.to_integer();
            if let Some(ctx) = context {
                ctx.actions.set_pos(x as i16, y as i16);
                ctx.user_pos_x = x as i16;
                ctx.user_pos_y = y as i16;
            }
            Ok(())
        }
        "LAUNCHAPP" => {
            let url = vm.pop("LAUNCHAPP")?.to_string();
            if let Some(ctx) = context {
                ctx.actions.launch_app(&url);
            }
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
