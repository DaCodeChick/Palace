# Ghidra MCP Bridge Workflow

## ⚠️ IMPORTANT: Clean Room Policy

**This project uses a strict clean-room implementation approach. Original Palace source code has been reviewed for algorithm verification ONLY and must NOT be distributed or directly referenced in our codebase.**

## 🔧 RDSS Policy: Refactor, Despaghettify, Simplify, Split

**When working on this codebase, always apply the RDSS principle:**

### Refactor
- Continuously improve code structure and organization
- Replace awkward APIs with ergonomic ones
- Eliminate unnecessary complexity and indirection
- Update code to use modern Rust patterns and idioms

### Despaghettify
- Break up tangled dependencies and circular references
- Separate concerns into distinct, focused components
- Make data flow clear and unidirectional where possible
- Eliminate global state and hidden dependencies

### Simplify
- Remove unused code, redundant logic, and unnecessary abstractions
- Use primitive types directly instead of type aliases that provide no safety
- Prefer explicit over implicit behavior
- Reduce cognitive load for future maintainers

### Split
- Break large modules into smaller, focused sub-modules
- When a file exceeds ~500 lines, consider splitting it
- Group related functionality into coherent modules
- Keep public APIs in parent modules, implementation details in sub-modules

**Examples of RDSS in action:**
- ✅ Removed `UserID`/`RoomID`/`HotspotID` type aliases → use `i32`/`i16` directly
- ✅ Removed `reserved` struct fields → handle padding in I/O operations only
- ✅ Changed `AuxFlags` from `i32` to `u32` → bitflags should always be unsigned
- ✅ Removed `UNUSED_1` flag → eliminate dead code

**When to apply RDSS:**
- During code review of new features
- When touching existing code for bug fixes
- When implementing new protocol features
- When you notice complexity that could be simplified
- Proactively, not just reactively

### What We've Extracted (Clean Room):

✅ **Algorithm Verification Completed (DO NOT reference original source again):**
- CRC32 algorithm: Rotate-left-with-carry + XOR (verified and implemented)
- Encryption: XOR cipher with 512-byte lookup table from seed 666666 (verified and implemented)
- Random Number Generator: Park-Miller PRNG (verified and implemented)
- CRC mask table: 256 u32 entries (extracted and implemented)

**All references to the original Mansion source code (`/home/admin/Downloads/mansionsrc/`) should now be avoided. The algorithms have been independently implemented in `lib/thepalace/src/algo.rs`.**

### Going Forward:

For any remaining protocol questions, use ONLY:
1. **Protocol PDF** (`/home/admin/Downloads/PalaceProtocolRef.pdf`) - primary reference
2. **Ghidra MCP analysis** - for binary behavior analysis (if needed)
3. **Network packet capture** - testing against original clients
4. **Independent implementation** - based on protocol spec, not decompiled code

## Purpose

Use Ghidra MCP server to decompile and analyze original Palace server and client binaries to understand:
- Protocol implementation details
- Iptscrae VM execution logic
- Undocumented message types or behaviors
- Prop format parsing algorithms
- Script security implementations
- Edge cases and quirks

**CRITICAL: Analyze for understanding, implement independently.**

## Setup

1. **Install Ghidra MCP Server**
   ```bash
   # Follow Ghidra MCP installation instructions
   # Configure MCP server to point at Palace binaries
   ```

2. **Locate Palace Binaries**
   - PalaceServer (Mac/Windows versions)
   - The Palace Client (Mac/Windows versions)
   - PalaceChat extensions
   - Phalanx modifications

3. **Import into Ghidra**
   ```bash
   # Create Ghidra project
   # Import binaries with auto-analysis enabled
   # Wait for analysis to complete
   ```

## Common Analysis Queries

### Protocol Constants

**Goal:** Find all message type definitions and verify against protocol PDF

```
Query: "Find all 4-byte integer constants between 0x60000000 and 0x80000000"
Query: "Show code references to 0x74697972 (MSG_TIYID)"
Query: "List all case statements in message dispatch function"
```

**Expected Output:** Message handler switch/case statements, constant definitions

**Action:** Cross-reference with protocol PDF, implement any missing message types

### Iptscrae VM

**Goal:** Understand stack-based execution model

```
Query: "Find function containing string 'SAY' or 'CHAT'"
Query: "Decompile Iptscrae bytecode interpreter loop"
Query: "Find stack push/pop operations"
Query: "List all Iptscrae opcode constants"
```

**Expected Output:** VM loop, opcode handlers, stack manipulation code

**Action:** Implement equivalent VM in Rust based on decompiled logic

### Prop Format Parsing

**Goal:** Understand 8-bit, 20-bit, 32-bit, S20-bit prop formats

```
Query: "Find code reading prop header (0x50726F70 'Prop')"
Query: "Decompile prop bitmap decoding function"
Query: "Show bit manipulation for 20-bit color conversion"
```

**Expected Output:** Prop parsing code, color conversion algorithms

**Action:** Implement prop decoders in Rust `assets/bitmap.rs`

### Encryption/CRC32

**Goal:** Verify encryption and CRC algorithms

```
Query: "Find CRC32 lookup table initialization"
Query: "Decompile string encryption function"
Query: "Show password hashing implementation"
```

**Expected Output:** Algorithm implementations, lookup tables

**Action:** Verify against existing implementation in `algo.rs`

### Cyborg Script Security

**Goal:** Understand client-side script sandboxing

```
Query: "Find 'cyborg' string references"
Query: "Show script permission checking code"
Query: "Decompile script timeout/limit enforcement"
```

**Expected Output:** Security checks, instruction limits, timeout code

**Action:** Implement equivalent sandboxing in `iptscrae/security.rs`

### Room Format

**Goal:** Parse RoomRec, Hotspot, LPropRec structures

```
Query: "Find RoomRec structure definition"
Query: "Decompile hotspot parsing code"
Query: "Show loose prop deserialization"
```

**Expected Output:** Structure layouts, parsing logic

**Action:** Implement in `room/parser.rs`

## Workflow Process

### 1. Identify Gap in Knowledge

Example: "How does the server handle MSG_DRAW commands?"

### 2. Search Ghidra

```
Query: "Find function handling message type 0x64726177"
Query: "Show code path from message dispatch to drawing handler"
```

### 3. Analyze Decompiled Code

```c
// Example decompiled output
void handle_draw_message(ClientMsg *msg) {
    DrawCommand *cmd = (DrawCommand *)msg->data;
    if (cmd->type == DRAW_LINE) {
        // ...
    }
}
```

### 4. Implement in Rust

```rust
// Equivalent Rust implementation
fn handle_draw_message(msg: &ClientMsg) -> Result<()> {
    let cmd = DrawCommand::from_bytes(&msg.data)?;
    match cmd.draw_type {
        DrawType::Line => { /* ... */ }
    }
}
```

### 5. Test Against Original Client

```bash
# Start our server
./palace-server

# Connect original Palace client
# Verify behavior matches
```

### 6. Document Findings

Update `docs/PROTOCOL.md` with any discoveries:
```markdown
## MSG_DRAW (0x64726177 'draw')

**Behavior:** Discovered via Ghidra analysis...
**Quirks:** Original server ignores alpha channel...
```

## Common Investigation Scenarios

### Scenario 1: Unspecified Protocol Behavior

**Problem:** Protocol PDF says "flags are various flag bits" with no details

**Investigation:**
1. Find flag usage in Ghidra
2. Identify bit positions and meanings
3. Document actual behavior
4. Implement in Rust

### Scenario 2: Iptscrae Function Missing Documentation

**Problem:** Need to implement `GETFACE` Iptscrae function, but behavior unclear

**Investigation:**
1. Search for "GETFACE" string in Ghidra
2. Find function implementation
3. Understand return value format
4. Implement in `iptscrae/stdlib.rs`

### Scenario 3: Edge Case Handling

**Problem:** What happens when user sends oversized prop?

**Investigation:**
1. Find prop receiving code
2. Check size validation
3. Find error handling path
4. Replicate validation logic

### Scenario 4: Extension Protocol

**Problem:** PalaceChat extension behavior undocumented

**Investigation:**
1. Load PalaceChat binary into Ghidra
2. Find extension registration code
3. Identify new message types
4. Document extension protocol
5. Implement compatibility layer

## Tips for Effective Analysis

### Naming Conventions
- Message handlers often named `handle_*` or `process_*`
- Message type constants often `MSG_*` or `kMsg*`
- Structures often have Hungarian notation: `pRoom`, `lpProp`

### Code Patterns
- Switch statements for message dispatch
- Byte-swapping code indicates network protocol
- Lookup tables suggest encryption/CRC
- `malloc`/`free` pairs indicate dynamic structures

### Cross-Referencing
- Find string constants, trace back to usage
- Find structure offsets, determine layout
- Find function calls, understand call graph

### Validation
- Compare decompiled code with protocol PDF
- Test edge cases against original server/client
- Check multiple binary versions for consistency

## MCP Query Examples

### Find Message Handler

```
Query: "Find the function that handles message type 0x74616c6b (MSG_TALK)"
Expected: Function pointer in dispatch table or switch case
```

### Extract Structure Definition

```
Query: "Show the structure layout for RoomRec based on memory access patterns"
Expected: Field offsets and types
```

### Trace Data Flow

```
Query: "Trace how user password flows from MSG_AUTHRESPONSE to database check"
Expected: Call graph showing authentication flow
```

### Find Hidden Features

```
Query: "List all message types defined but not documented in protocol PDF"
Expected: Unused or undocumented message constants
```

## Documentation Standards

When documenting findings from Ghidra analysis:

1. **Reference Binary:** "Analyzed in PalaceServer v3.5 (Mac)"
2. **Function Location:** "Function at offset 0x12345"
3. **Confidence Level:** "High confidence" / "Needs verification"
4. **Cross-Reference:** "Matches protocol PDF section 3.43"
5. **Implementation Note:** "Implemented in server/src/net/handler.rs:123"

## Safety Considerations

- **Reverse Engineering:** Only for interoperability, not redistribution
- **Clean Room:** Understand behavior, implement independently
- **Testing:** Verify compatibility without copying code
- **Documentation:** Reference findings, not decompiled code directly

## Integration with Development

### During Protocol Implementation

```bash
# When implementing new message type
1. Read protocol PDF section
2. Query Ghidra for actual implementation
3. Note any discrepancies
4. Implement based on actual behavior
5. Test with original client
```

### During Debugging

```bash
# When behavior doesn't match expectations
1. Reproduce issue with original server
2. Find relevant code in Ghidra
3. Understand expected behavior
4. Fix our implementation
5. Verify fix works with original client
```

### During Extension Development

```bash
# When adding PalaceChat/Phalanx support
1. Load extension binary in Ghidra
2. Find extension initialization code
3. Document new message types
4. Implement compatibility layer
5. Test with extension-enabled client
```

## Conclusion

The Ghidra MCP bridge is invaluable for:
- Filling gaps in protocol documentation
- Understanding undocumented behavior
- Ensuring compatibility with original clients
- Discovering hidden features or quirks
- Validating our implementation

Always cross-reference findings with the protocol PDF and test against original Palace software for maximum compatibility.
