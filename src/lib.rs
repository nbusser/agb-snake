#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

mod snake;

use agb::{
    display::{
        self,
        tiled::{RegularMap, TileFormat, TileSetting, TiledMap, VRamManager},
        Priority,
    },
    include_background_gfx,
    input::Button,
    sound::mixer::Frequency,
};

include_background_gfx!(backgrounds, "121105",
    background => deduplicate "gfx/background.png",
    grid => deduplicate "gfx/grid.aseprite",
);

fn create_background(background: &mut RegularMap, vram: &mut VRamManager) {
    background.set_scroll_pos((0i16, 0));
    vram.set_background_palettes(backgrounds::PALETTES);
    background.set_visible(false);
    background.fill_with(vram, &backgrounds::background);
    background.commit(vram);
    background.set_visible(true);
}

fn create_grid(grid: &mut RegularMap, vram: &mut VRamManager) {
    #[repr(u16)]
    enum GridTileId {
        Plain = 0,
        Corner = 1,
        Roof = 2,
        Wall = 3,
    }

    const MIN_X: u16 = 4;
    const MIN_Y: u16 = 4;
    const MAX_X: u16 = 26;
    const MAX_Y: u16 = 16;

    for x in MIN_X..MAX_X {
        for y in MIN_Y..MAX_Y {
            let mut tile_id = GridTileId::Plain;
            let mut hflip = false;
            let mut vflip = false;

            if (x == MIN_X || x == MAX_X - 1) && (y == MIN_Y || y == MAX_Y - 1) {
                tile_id = GridTileId::Corner;
                hflip = x == MAX_X - 1;
                vflip = y == MAX_Y - 1;
            } else if x == MIN_X || x == MAX_X - 1 {
                tile_id = GridTileId::Wall;
                hflip = x == MAX_X - 1;
            } else if y == MIN_Y || y == MAX_Y - 1 {
                tile_id = GridTileId::Roof;
                vflip = y == MAX_Y - 1;
            }

            grid.set_tile(
                vram,
                (x, y),
                &backgrounds::grid.tiles,
                TileSetting::new(tile_id as u16, hflip, vflip, 0),
            );
        }
    }
    grid.commit(vram);
    grid.set_visible(true);
}

pub fn main(mut gba: agb::Gba) -> ! {
    let objects = gba.display.object.get_managed();
    let vblank = agb::interrupt::VBlank::get();

    let (tiled, mut vram) = gba.display.video.tiled0();
    let mut background = tiled.background(
        Priority::P3,
        display::tiled::RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut background_grid = tiled.background(
        Priority::P2,
        display::tiled::RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let mut snake = snake::Snake::new(3, &objects);

    loop {
        create_background(&mut background, &mut vram);

        let mut input = agb::input::ButtonController::new();
        loop {
            input.update();
            if input.is_just_pressed(agb::input::Button::all()) {
                break;
            }
            vblank.wait_for_vblank();
        }

        create_grid(&mut background_grid, &mut vram);

        loop {
            while snake.is_alive {
                for _n_frames in 0..30 {
                    input.update();

                    if input.is_just_pressed(agb::input::Button::UP) {
                        snake.apply_input(Button::UP);
                    } else if input.is_just_pressed(agb::input::Button::DOWN) {
                        snake.apply_input(Button::DOWN);
                    } else if input.is_just_pressed(agb::input::Button::LEFT) {
                        snake.apply_input(Button::LEFT);
                    } else if input.is_just_pressed(agb::input::Button::RIGHT) {
                        snake.apply_input(Button::RIGHT);
                    }
                    vblank.wait_for_vblank();
                }
                snake.try_move(&objects);
                snake.display(&objects);
            }
        }
    }
}
