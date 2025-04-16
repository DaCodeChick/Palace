use bitflags::bitflags;

const PROP: u32 = 0x50726F70;

bitflags! {
    /// Characterizes how a prop behaves. The server does not use these.
    pub struct PropFlags: u32 {
        const FORMAT_8BIT = 0;
        const HEAD = 2;
        const GHOST = 4;
        const RARE = 8;
        const ANIMATE = 16;
        const BOUNCE = 32;
        const FORMAT_20BIT = 64;
        const FORMAT_32BIT = 256;
        const FORMAT_S20BIT = 512;
        const FORMAT_MASK = FORMAT_20BIT | FORMAT_32BIT | FORMAT_S20BIT;
        const PNG = 1024;
        const LEGACY = 0xFFC1;
    }
}
