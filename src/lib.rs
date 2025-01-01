#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

mod apple;
mod constants;
mod snake;

use agb::{
    display::{
        self,
        tiled::{RegularMap, TileFormat, TileSetting, TiledMap, VRamManager},
        Priority,
    },
    include_background_gfx,
    input::Button,
    println,
    rng::RandomNumberGenerator,
    sound::mixer::Frequency,
};

include_background_gfx!(backgrounds, "121105",
    background => deduplicate "gfx/plain-background.png",
    props => deduplicate "gfx/props.aseprite",
);

fn create_background(
    background: &mut RegularMap,
    props: &mut RegularMap,
    vram: &mut VRamManager,
    rng: &mut RandomNumberGenerator,
) {
    background.set_scroll_pos((0i16, 0));
    vram.set_background_palettes(backgrounds::PALETTES);

    background.fill_with(vram, &backgrounds::background);
    background.commit(vram);
    background.set_visible(true);

    const NB_PROPS_TILES: i32 = 3;
    const N_TILES_X: u32 = (display::WIDTH / constants::TILE_SIZE) as u32;
    const N_TILES_Y: u32 = (display::HEIGHT / constants::TILE_SIZE) as u32;
    for x in 0..N_TILES_X {
        for y in 0..N_TILES_Y {
            if rng.gen().rem_euclid(100) < 2 {
                let tile_id = rng.gen().rem_euclid(NB_PROPS_TILES) as u16;
                props.set_tile(
                    vram,
                    (x as u16, y as u16),
                    &backgrounds::props.tiles,
                    TileSetting::new(tile_id, false, false, 0),
                );
            };
        }
    }
    props.commit(vram);
    props.set_visible(true);
}

pub fn main(mut gba: agb::Gba) -> ! {
    let objects = gba.display.object.get_managed();
    let vblank = agb::interrupt::VBlank::get();

    let (tiled, mut vram) = gba.display.video.tiled1();
    let mut background = tiled.regular(
        Priority::P1,
        display::tiled::RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut props = tiled.regular(
        Priority::P0,
        display::tiled::RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let mut rng = RandomNumberGenerator::new();

    create_background(&mut background, &mut props, &mut vram, &mut rng);

    let mut apple = apple::Apple::new(&objects, &mut rng);
    let mut snake = snake::Snake::new(3, &objects);

    loop {
        let mut input = agb::input::ButtonController::new();
        loop {
            input.update();
            if input.is_just_pressed(agb::input::Button::all()) {
                break;
            }
            vblank.wait_for_vblank();
        }

        loop {
            while snake.is_alive {
                let mut next_input = None;
                for _n_frames in 0..15 {
                    input.update();

                    if let Some(frame_input) =
                        [Button::UP, Button::DOWN, Button::LEFT, Button::RIGHT]
                            .iter()
                            .find(|button| input.is_just_pressed(**button))
                            .copied()
                    {
                        next_input = Some(frame_input);
                    };

                    vblank.wait_for_vblank();
                }

                if let Some(input) = next_input {
                    snake.apply_input(input);
                }

                snake.try_move(&objects, &mut apple);
                objects.commit();
            }
        }
    }
}
