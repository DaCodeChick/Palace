use bitflags::bitflags;

const PROP: u32 = 0x50726F70;

bitflags! {
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
		const FORMAT_MASK = 0x340;
		const PNG = 1024;
		const LEGACY = 0xFFC1;
	}
}
