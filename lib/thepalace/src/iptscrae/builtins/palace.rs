//! Palace-specific builtin functions for Iptscrae VM.

use crate::iptscrae::context::{ScriptContext, SecurityLevel};
use crate::iptscrae::value::Value;
use crate::iptscrae::vm::{Vm, VmError};

/// Execute Palace-specific builtin functions.
pub fn execute_palace_builtin(
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
        // Prop manipulation functions
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
        "MACRO" => {
            // Execute a macro (prop script) - stub for now
            let _macro_id = vm.pop("MACRO")?.to_integer();
            // Would need to look up and execute the macro script
            Ok(())
        }
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
        "GOTOROOM" => {
            vm.require_permission(context.as_deref(), "GOTOROOM")?;
            let room_id = vm.pop("GOTOROOM")?.to_integer();
            vm.with_context_action(context, |ctx| ctx.actions.goto_room(room_id as i16));
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
        // Position/Movement functions
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
        // Room/Navigation functions
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
        // User Info functions
        "USERID" => {
            // USERID is an alias for WHOME
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_id));
            } else {
                vm.push(Value::Integer(0));
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
        // Message functions
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
        // Door Commands
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
        "DEST" => {
            // Get destination room ID for a door - would need room data
            let _door_id = vm.pop("DEST")?.to_integer();
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
        // Spot Commands
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
        "SETPICLOC" => {
            // Set picture location (for drawing) - would need graphics context
            let _y = vm.pop("SETPICLOC y")?.to_integer();
            let _x = vm.pop("SETPICLOC x")?.to_integer();
            // For now, just consume the parameters
            Ok(())
        }
        // Loose Props
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
        // Server/System functions
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
        "MOUSEPOS" => {
            // Get mouse position - would need UI state
            // Push X and Y coordinates
            vm.push(Value::Integer(0));
            vm.push(Value::Integer(0));
            Ok(())
        }
        "DELAY" => {
            // Delay execution - not implemented (would need async/timer support)
            let _milliseconds = vm.pop("DELAY")?.to_integer();
            Ok(())
        }
        "DIMROOM" => {
            // Dim the room - UI effect
            let _brightness = vm.pop("DIMROOM")?.to_integer();
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
        // Special functions
        "ME" => {
            // Return current user ID
            if let Some(ctx) = context {
                vm.push(Value::Integer(ctx.user_id));
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
        // Sound/Media functions
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
        "LAUNCHAPP" => {
            let url = vm.pop("LAUNCHAPP")?.to_string();
            if let Some(ctx) = context {
                ctx.actions.launch_app(&url);
            }
            Ok(())
        }
        // Painting/Graphics functions (stubs - would need graphics context)
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
