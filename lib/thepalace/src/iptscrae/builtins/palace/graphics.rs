//! Graphics builtin functions for Palace.

use crate::iptscrae::context::ScriptContext;
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute graphics builtin functions.
pub fn execute_graphics_builtin(
    vm: &mut Vm,
    name: &str,
    context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    match name {
        "POSX" => {
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_pos_x as i32));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "POSY" => {
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_pos_y as i32));
            } else {
                vm.push(Value::Integer(0));
            }
            Ok(())
        }
        "SETPOS" => {
            let y = vm.pop("SETPOS y")?.to_integer();
            let x = vm.pop("SETPOS x")?.to_integer();
            if let Some(ctx) = context {
                ctx.actions.set_pos(x as i16, y as i16);
                ctx.user_pos_x = x as i16;
                ctx.user_pos_y = y as i16;
            }
            Ok(())
        }
        "LINE" => {
            // Draw line: x1 y1 x2 y2 -> draw line
            let _y2 = vm.pop("LINE y2")?.to_integer();
            let _x2 = vm.pop("LINE x2")?.to_integer();
            let _y1 = vm.pop("LINE y1")?.to_integer();
            let _x1 = vm.pop("LINE x1")?.to_integer();
            Ok(())
        }
        "LINETO" => {
            // Draw line to: x y -> draw line from pen position to x,y
            let _y = vm.pop("LINETO y")?.to_integer();
            let _x = vm.pop("LINETO x")?.to_integer();
            Ok(())
        }
        "PENPOS" => {
            // Get pen position - push X and Y
            vm.push(Value::Integer(0));
            vm.push(Value::Integer(0));
            Ok(())
        }
        "PENTO" => {
            // Set pen position
            let _y = vm.pop("PENTO y")?.to_integer();
            let _x = vm.pop("PENTO x")?.to_integer();
            Ok(())
        }
        "PENSIZE" => {
            // Set pen size
            let _size = vm.pop("PENSIZE")?.to_integer();
            Ok(())
        }
        "PENCOLOR" => {
            // Set pen color
            let _color = vm.pop("PENCOLOR")?.to_integer();
            Ok(())
        }
        "PENFRONT" => {
            // Set pen to draw in front
            Ok(())
        }
        "PENBACK" => {
            // Set pen to draw in back
            Ok(())
        }
        "PAINTCLEAR" => {
            // Clear all painting
            Ok(())
        }
        "PAINTUNDO" => {
            // Undo last painting operation
            Ok(())
        }
        _ => Err(VmError::UndefinedFunction {
            name: name.to_string(),
        }),
    }
}
