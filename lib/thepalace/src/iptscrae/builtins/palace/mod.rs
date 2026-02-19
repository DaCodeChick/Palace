//! Palace-specific builtin functions organized by category.

mod graphics;
mod messaging;
mod navigation;
mod props;
mod room;
mod system;
mod user;

use crate::iptscrae::context::ScriptContext;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute Palace-specific builtin functions.
///
/// This function dispatches to category-specific modules:
/// - messaging: SAY, CHAT, LOCALMSG, ROOMMSG, PRIVATEMSG, etc.
/// - props: GETPROPS, SETPROPS, NAKED, DONPROP, etc.
/// - user: USERNAME, WHOME, SETFACE, SETCOLOR, etc.
/// - navigation: GOTOROOM, GOTOURL, NETGOTO, etc.
/// - room: ROOMNAME, ROOMID, NBRDOORS, LOCK, UNLOCK, etc.
/// - graphics: PENCOLOR, LINE, LINETO, PAINTCLEAR, etc.
/// - system: DELAY, BEEP, SOUND, TICKS, DATETIME, etc.
pub fn execute_palace_builtin(
    vm: &mut Vm,
    name: &str,
    mut context: Option<&mut ScriptContext>,
) -> Result<(), VmError> {
    // Try messaging functions
    match messaging::execute_messaging_builtin(vm, name, context.as_deref_mut()) {
        Ok(()) => return Ok(()),
        Err(VmError::UndefinedFunction { .. }) => {}
        Err(e) => return Err(e),
    }

    // Try props functions
    match props::execute_props_builtin(vm, name, context.as_deref_mut()) {
        Ok(()) => return Ok(()),
        Err(VmError::UndefinedFunction { .. }) => {}
        Err(e) => return Err(e),
    }

    // Try user functions
    match user::execute_user_builtin(vm, name, context.as_deref_mut()) {
        Ok(()) => return Ok(()),
        Err(VmError::UndefinedFunction { .. }) => {}
        Err(e) => return Err(e),
    }

    // Try navigation functions
    match navigation::execute_navigation_builtin(vm, name, context.as_deref_mut()) {
        Ok(()) => return Ok(()),
        Err(VmError::UndefinedFunction { .. }) => {}
        Err(e) => return Err(e),
    }

    // Try room functions
    match room::execute_room_builtin(vm, name, context.as_deref_mut()) {
        Ok(()) => return Ok(()),
        Err(VmError::UndefinedFunction { .. }) => {}
        Err(e) => return Err(e),
    }

    // Try graphics functions
    match graphics::execute_graphics_builtin(vm, name, context.as_deref_mut()) {
        Ok(()) => return Ok(()),
        Err(VmError::UndefinedFunction { .. }) => {}
        Err(e) => return Err(e),
    }

    // Try system functions
    system::execute_system_builtin(vm, name, context)
}
