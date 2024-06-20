// Constants that correspond to key presses

//associated constants
pub const ONE_BIT: u32 = 0x1;
pub const TWO_BIT: u32 = 0x3;
pub const THREE_BIT: u32 = 0x7;
pub const FOUR_BIT: u32 = 0xF;
pub const FIVE_BIT: u32 = 0x1F;
pub const SEVEN_BIT: u32 = 0x7F;
pub const EIGHT_BIT: u32 = 0xFF;
pub const TEN_BIT: u32 = 0x3FF;
pub const SIXTEEN_BIT: u32 = 0xFFFF;
pub const THIRTY_BIT: u32 = 0x3FFFFFFF;

// convert slices of u8 to one u32

// Big Endian
pub fn _to_u32_be(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 24)
        + ((bytes[1] as u32) << 16)
        + ((bytes[2] as u32) << 8)
        + (bytes[3] as u32)
}

// Little Endian
pub fn to_u32_le(bytes: &[u8]) -> u32 {
    (bytes[0] as u32)
        + ((bytes[1] as u32) << 8)
        + ((bytes[2] as u32) << 16)
        + ((bytes[3] as u32) << 24)
}
