use agb::display;

pub const TILE_SIZE: i32 = 8;
pub const OBJECTS_SIZE: i32 = 16;
pub const BOARD_MIN_X: i32 = 0;
pub const BOARD_MAX_X: i32 = (display::WIDTH / OBJECTS_SIZE) - 1;
pub const BOARD_MIN_Y: i32 = 0;
pub const BOARD_MAX_Y: i32 = (display::HEIGHT / OBJECTS_SIZE) - 1;
