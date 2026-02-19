# RDSS: Room Script to Protocol Converter

## Research

### Goal
Convert parsed room script AST structures (`RoomDecl`, `DoorDecl`, `SpotDecl`) into Palace protocol structures (`RoomRec`, `Hotspot`, `PictureRec`) that can be used as **room templates** for server initialization or storage.

**Important:** Room scripts define the **static/template** portion of a room. The resulting `RoomRec` will have:
- ✅ Static fields populated (id, name, flags, hotspots, pictures, scripts)
- ❌ Runtime fields zeroed (nbr_people=0, nbr_lprops=0, nbr_draw_cmds=0)

The server will later populate runtime fields as users interact with the room.

### Input Structures (AST - from room_script.rs)

**RoomDecl:**
```rust
pub struct RoomDecl {
    pub id: i16,                          // Room ID (required)
    pub name: Option<String>,             // Room name
    pub pict: Option<String>,             // Background picture filename
    pub artist: Option<String>,           // Artist name
    pub password: Option<String>,         // Room password
    pub flags: RoomFlags,                 // Boolean flags (5 fields)
    pub pictures: Vec<PictureDecl>,       // Additional picture layers
    pub doors: Vec<DoorDecl>,             // Door hotspots
    pub spots: Vec<SpotDecl>,             // Regular hotspots
}

pub struct RoomFlags {
    pub private: bool,        // RF_PRIVATE (0x0002)
    pub no_painting: bool,    // RF_NO_PAINTING (0x0004)
    pub no_cyborgs: bool,     // RF_CYBORG_FREE_ZONE (0x0010)
    pub hidden: bool,         // RF_HIDDEN (0x0020)
    pub no_guests: bool,      // RF_NO_GUESTS (0x0040)
}

pub struct PictureDecl {
    pub id: i16,
    pub name: String,
    pub trans_color: Option<i16>,
}

pub struct DoorDecl {
    pub id: i16,
    pub dest: i16,
    pub name: Option<String>,
    pub outline: Vec<Point>,
    pub picts: Vec<StateDecl>,
    pub script: Option<Script>,
}

pub struct SpotDecl {
    pub id: i16,
    pub name: Option<String>,
    pub outline: Vec<Point>,
    pub picts: Vec<StateDecl>,
    pub script: Option<Script>,
}

pub struct StateDecl {
    pub pic_id: i16,
    pub x_offset: i16,
    pub y_offset: i16,
}
```

### Output Structures (Protocol - from messages/room/records.rs)

**RoomRec:**
```rust
pub struct RoomRec {
    pub room_flags: RoomFlags,       // Bitflags (u16)
    pub faces_id: i32,               // Default face ID
    pub room_id: i16,
    pub room_name_ofst: i16,         // Offset into varBuf
    pub pict_name_ofst: i16,         // Offset into varBuf
    pub artist_name_ofst: i16,       // Offset into varBuf
    pub password_ofst: i16,          // Offset into varBuf
    pub nbr_hotspots: i16,
    pub hotspot_ofst: i16,           // 4-byte aligned
    pub nbr_pictures: i16,
    pub picture_ofst: i16,           // 4-byte aligned
    pub nbr_draw_cmds: i16,
    pub first_draw_cmd: i16,
    pub nbr_people: i16,
    pub nbr_lprops: i16,
    pub first_lprop: i16,
    pub len_vars: i16,
    pub var_buf: Bytes,              // Variable-length data
}

pub struct Hotspot {
    pub script_event_mask: EventMask,  // i32
    pub flags: i32,
    pub secure_info: i32,
    pub ref_con: i32,
    pub loc: Point,
    pub id: i16,
    pub dest: i16,                     // Only for doors
    pub nbr_pts: i16,
    pub pts_ofst: i16,                 // Offset into varBuf
    pub hotspot_type: HotspotType,     // Door vs Normal
    pub group_id: i16,
    pub nbr_scripts: i16,
    pub script_rec_ofst: i16,          // Offset into varBuf
    pub state: HotspotState,
    pub nbr_states: i16,
    pub state_rec_ofst: i16,           // Offset into varBuf
    pub name_ofst: i16,                // Offset into varBuf
    pub script_text_ofst: i16,         // Offset into varBuf
}

pub struct PictureRec {
    pub ref_con: i32,
    pub pic_id: i16,
    pub pic_name_ofst: i16,            // Offset into varBuf
    pub trans_color: i16,
}
```

### Key Discoveries

1. **varBuf Architecture**: Protocol structures use a single `var_buf: Bytes` to store all variable-length data (strings, arrays). Fixed-size fields contain offsets pointing into this buffer.

2. **Alignment Requirements**: Arrays (hotspots, pictures, points, states) must be 4-byte aligned within varBuf.

3. **String Format**: All strings in varBuf are stored as PStrings (length byte + data).

4. **RoomFlags Mismatch**: 
   - AST has 5 boolean fields
   - Protocol uses bitflags (u16) with 10+ possible flags
   - Mapping: AST bools → protocol bits

5. **EventMask Extraction**: Scripts contain event handlers (ON SELECT, ON ENTER, etc.). Must extract event types from Script to build `script_event_mask`.

6. **Hotspot Type Detection**: DoorDecl always maps to HotspotType::Door, SpotDecl always maps to HotspotType::Normal.

7. **Missing Fields**: Protocol structures have fields not in AST:
   - `faces_id` (default: 0)
   - `flags`, `secure_info`, `ref_con` for hotspots (default: 0)
   - `group_id`, `state` (default: 0 = Unlocked)
   - `nbr_people`, `nbr_lprops`, `first_lprop`, `nbr_draw_cmds`, `first_draw_cmd` (default: 0)

## Design

### Field Mapping

#### RoomDecl → RoomRec

| AST Field | Protocol Field | Conversion |
|-----------|---------------|------------|
| `id` | `room_id` | Direct copy |
| `name` | `room_name_ofst` + varBuf | Write PString to varBuf, store offset |
| `pict` | `pict_name_ofst` + varBuf | Write PString to varBuf, store offset (or -1) |
| `artist` | `artist_name_ofst` + varBuf | Write PString to varBuf, store offset (or -1) |
| `password` | `password_ofst` + varBuf | Write PString to varBuf, store offset (or -1) |
| `flags.private` | `room_flags & PRIVATE` | Convert to bitflag |
| `flags.no_painting` | `room_flags & NO_PAINTING` | Convert to bitflag |
| `flags.no_cyborgs` | `room_flags & CYBORG_FREE_ZONE` | Convert to bitflag |
| `flags.hidden` | `room_flags & HIDDEN` | Convert to bitflag |
| `flags.no_guests` | `room_flags & NO_GUESTS` | Convert to bitflag |
| `pictures.len()` | `nbr_pictures` | Count |
| `pictures` | `picture_ofst` + varBuf | Write array, 4-byte aligned |
| `doors.len() + spots.len()` | `nbr_hotspots` | Count |
| `doors + spots` | `hotspot_ofst` + varBuf | Write array, 4-byte aligned |
| - | `faces_id` | Default: 0 |
| - | `nbr_draw_cmds` | Default: 0 |
| - | `first_draw_cmd` | Default: 0 |
| - | `nbr_people` | Default: 0 |
| - | `nbr_lprops` | Default: 0 |
| - | `first_lprop` | Default: 0 |

#### DoorDecl → Hotspot

| AST Field | Protocol Field | Conversion |
|-----------|---------------|------------|
| `id` | `id` | Direct copy |
| `dest` | `dest` | Direct copy |
| `name` | `name_ofst` + varBuf | Write PString to varBuf |
| `outline` | `nbr_pts` + `pts_ofst` + varBuf | Count + write Point array |
| `picts` | `nbr_states` + `state_rec_ofst` + varBuf | Count + write state array |
| `script` | `script_event_mask` + `nbr_scripts` + `script_rec_ofst` + `script_text_ofst` | Extract event mask, write script |
| - | `hotspot_type` | Always `HotspotType::Door` (1) |
| - | `loc` | First point from outline or (0,0) |
| - | `flags` | Default: 0 |
| - | `secure_info` | Default: 0 |
| - | `ref_con` | Default: 0 |
| - | `group_id` | Default: 0 |
| - | `state` | Default: `HotspotState::Unlocked` (0) |

#### SpotDecl → Hotspot

Same as DoorDecl except:
- `dest` = 0 (no destination)
- `hotspot_type` = `HotspotType::Normal` (0)

#### PictureDecl → PictureRec

| AST Field | Protocol Field | Conversion |
|-----------|---------------|------------|
| `id` | `pic_id` | Direct copy |
| `name` | `pic_name_ofst` + varBuf | Write PString to varBuf |
| `trans_color` | `trans_color` | Copy or default -1 |
| - | `ref_con` | Default: 0 |

### Conversion Challenges

1. **varBuf Construction**: Need to:
   - Track current offset while writing
   - Align offsets to 4-byte boundaries for arrays
   - Write PStrings for all strings
   - Write arrays of fixed-size structs (Hotspot, PictureRec, Point, StateRec)
   - Return final Bytes buffer

2. **Event Mask Extraction**: Given `Script`, iterate over `handlers` and collect all `EventType` values, then convert to `EventMask` bitflags.

3. **Script Serialization**: Need to serialize Script AST back to Iptscrae source text for `script_text_ofst`.

4. **State Records**: StateDecl (AST) → Need protocol structure. Check if exists in codebase, or define as:
   ```rust
   struct StateRec {
       pic_id: i16,
       x_offset: i16,
       y_offset: i16,
   }
   ```

5. **Script Records**: Need protocol structure for `script_rec_ofst`. Likely:
   ```rust
   struct ScriptRec {
       event_type: i16,
       script_text_ofst: i16,
   }
   ```

## Specification

### API Design

```rust
#[cfg(feature = "room-script")]
pub mod room_script_converter;

// Main conversion function
pub fn convert_room(room: &RoomDecl) -> Result<RoomRec, ConversionError>;

// Helper functions
fn build_var_buf(room: &RoomDecl) -> VarBufBuilder;
fn convert_flags(flags: &room_script::RoomFlags) -> messages::flags::RoomFlags;
fn convert_door(door: &DoorDecl, var_buf: &mut VarBufBuilder) -> Result<Hotspot, ConversionError>;
fn convert_spot(spot: &SpotDecl, var_buf: &mut VarBufBuilder) -> Result<Hotspot, ConversionError>;
fn convert_picture(pic: &PictureDecl, var_buf: &mut VarBufBuilder) -> Result<PictureRec, ConversionError>;
fn extract_event_mask(script: &Script) -> EventMask;
fn serialize_script(script: &Script) -> String;
```

### VarBufBuilder

Helper struct to manage varBuf construction:

```rust
struct VarBufBuilder {
    buf: BytesMut,
    current_offset: usize,
}

impl VarBufBuilder {
    fn new() -> Self;
    
    // Write PString, return offset
    fn write_pstring(&mut self, s: &str) -> i16;
    
    // Write PString or return -1 if None
    fn write_optional_pstring(&mut self, s: Option<&str>) -> i16;
    
    // Align to 4-byte boundary
    fn align_to_4(&mut self);
    
    // Write fixed-size value, return offset
    fn write_struct<T: ToBytes>(&mut self, value: &T) -> i16;
    
    // Write array of fixed-size values, return offset
    fn write_array<T: ToBytes>(&mut self, values: &[T]) -> i16;
    
    // Write Point array
    fn write_points(&mut self, points: &[Point]) -> i16;
    
    // Write StateRec array
    fn write_states(&mut self, states: &[StateDecl]) -> i16;
    
    // Get final buffer
    fn finish(self) -> Bytes;
}
```

### Error Handling

```rust
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// varBuf would exceed i16::MAX (32767 bytes)
    VarBufTooLarge { size: usize },
    
    /// Too many hotspots (max i16::MAX)
    TooManyHotspots { count: usize },
    
    /// Too many pictures (max i16::MAX)
    TooManyPictures { count: usize },
    
    /// Too many points in outline (max i16::MAX)
    TooManyPoints { hotspot_id: i16, count: usize },
    
    /// Too many states (max i16::MAX)
    TooManyStates { hotspot_id: i16, count: usize },
    
    /// String too long for PString (max 255 bytes)
    StringTooLong { field: String, length: usize },
    
    /// Script serialization failed
    ScriptSerializationError { message: String },
}

impl std::fmt::Display for ConversionError { ... }
impl std::error::Error for ConversionError {}
```

### Conversion Algorithm

```
1. Create VarBufBuilder
2. Convert room flags (AST bools → protocol bitflags)
3. Write room strings to varBuf (name, pict, artist, password), record offsets
4. Align to 4-byte boundary
5. Write pictures array:
   - For each PictureDecl:
     - Write picture name to varBuf
     - Create PictureRec with offset
   - Write PictureRec array to varBuf (4-byte aligned)
   - Record picture_ofst
6. Align to 4-byte boundary
7. Write hotspots array:
   - For each DoorDecl + SpotDecl:
     - Write name to varBuf
     - Write outline points array to varBuf (4-byte aligned)
     - Write states array to varBuf (4-byte aligned)
     - If script exists:
       - Extract event mask
       - Serialize script to text
       - Write script text to varBuf
     - Create Hotspot with all offsets
   - Write Hotspot array to varBuf (4-byte aligned)
   - Record hotspot_ofst
8. Get final varBuf bytes
9. Create RoomRec with all fields set
10. Return RoomRec
```

## Solution Plan

### Module Placement

**Where should this converter live?**

The converter bridges AST → Protocol structures, so it could go in either:

**Option A: `src/iptscrae/room_script_converter.rs`**
- ✅ Close to the parser and AST definitions
- ✅ All room script code in one module
- ✅ Feature-gated together: `#[cfg(feature = "room-script")]`
- ❌ Mixes parsing (AST) with domain logic (conversion)

**Option B: `src/room/converter.rs`**
- ✅ Separates parsing from conversion logic
- ✅ `room/` module focuses on domain logic
- ✅ Better separation of concerns
- ❌ Requires both features: `#[cfg(all(feature = "room", feature = "room-script"))]`
- ❌ `room/` module currently minimal (just enums)

**Decision: Option A** - Keep in `iptscrae/room_script_converter.rs` for now because:
1. The `room/` module is currently minimal and focused on enums
2. Keeps all room script functionality together
3. Simpler feature gating
4. Can be refactored later if `room/` module grows

### Room Template vs Live Room

The converter produces a **room template** (static definition):
- All static fields populated from room script
- Runtime fields set to defaults:
  - `faces_id = 0` (or configurable default)
  - `nbr_people = 0`
  - `nbr_lprops = 0`, `first_lprop = 0`
  - `nbr_draw_cmds = 0`, `first_draw_cmd = 0`

The **server** is responsible for:
1. Loading room scripts → converting to templates
2. Instantiating live rooms from templates
3. Managing runtime state (user count, props, drawings)
4. Cloning templates when creating new room instances

### Implementation Steps

1. **Define protocol structures** (if missing):
   - Check if `StateRec` exists, define if needed
   - Check if `ScriptRec` exists, define if needed

2. **Implement VarBufBuilder**:
   - Core write methods
   - Alignment logic
   - Offset tracking

3. **Implement helper converters**:
   - `convert_flags()`
   - `extract_event_mask()`
   - `serialize_script()`

4. **Implement main converter**:
   - `convert_room()`
   - `convert_door()`
   - `convert_spot()`
   - `convert_picture()`

5. **Add unit tests**:
   - Test VarBufBuilder methods
   - Test flag conversion
   - Test event mask extraction
   - Test simple room conversion
   - Test room with doors and spots
   - Test room with pictures
   - Test error cases (too large, too many items)

6. **Add integration tests**:
   - Parse complete room script file
   - Convert to RoomRec
   - Verify all fields
   - Round-trip test (serialize + deserialize)

### Test Strategy

**Unit Tests:**
- VarBufBuilder alignment
- VarBufBuilder PString writing
- Flag conversion (all combinations)
- Event mask extraction (all event types)
- Simple conversions (single door, single spot, single picture)
- Edge cases (empty strings, max values)

**Integration Tests:**
- Full room with all features
- Multiple rooms
- Room with complex scripts
- Error cases (exceed limits)

### Open Questions

1. **StateRec format**: Need to confirm if this struct exists or needs definition
2. **ScriptRec format**: Need to confirm format for script records
3. **Script serialization**: Should we use a Script::to_string() method or custom serializer?
4. **loc field**: Should we use first point of outline, or compute centroid, or use (0,0)?

### Success Criteria

- [x] All unit tests pass
- [x] All integration tests pass
- [x] Parse full room script → Convert to RoomRec → Serialize to bytes → Deserialize → Verify
- [x] 0 clippy warnings
- [x] All tests pass with and without room-script feature
- [x] Documentation complete
