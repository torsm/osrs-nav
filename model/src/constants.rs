pub const HORIZONTAL_REGIONS: u32 = 100;
pub const VERTICAL_REGIONS: u32 = 200;
pub const PLANES: u32 = 4;
pub const REGION_SIZE: u32 = 64;

pub const WIDTH: u32 = HORIZONTAL_REGIONS * REGION_SIZE;
pub const HEIGHT: u32 = VERTICAL_REGIONS * REGION_SIZE;

pub const FLAG_N: u8 = 0x1;
pub const FLAG_E: u8 = 0x2;
pub const FLAG_S: u8 = 0x4;
pub const FLAG_W: u8 = 0x8;
pub const FLAG_NE: u8 = 0x10;
pub const FLAG_SE: u8 = 0x20;
pub const FLAG_SW: u8 = 0x40;
pub const FLAG_NW: u8 = 0x80;

pub const DIRECTIONS: [(u8, i32, i32); 8] = [
    (FLAG_W, -1, 0),
    (FLAG_E, 1, 0),
    (FLAG_NW, -1, 1),
    (FLAG_N, 0, 1),
    (FLAG_NE, 1, 1),
    (FLAG_SW, -1, -1),
    (FLAG_S, 0, -1),
    (FLAG_SE, 1, -1)
];
