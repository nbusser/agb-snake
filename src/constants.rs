use agb::display;

pub const TILE_SIZE: i32 = 16;
pub const MIN_X: i32 = 0;
pub const MAX_X: i32 = (display::WIDTH / TILE_SIZE) - 1;
pub const MIN_Y: i32 = 0;
pub const MAX_Y: i32 = (display::HEIGHT / TILE_SIZE) - 1;
