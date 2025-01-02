#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

mod apple;
mod background;
mod constants;
mod sfx;
mod snake;

use agb::{
    display::{self, tiled::TileFormat, Priority},
    input::{Button, ButtonController},
    interrupt::VBlank,
    rng::RandomNumberGenerator,
    sound::mixer::Frequency,
};
use sfx::Sfx;

fn wait_for_input(vblank: &VBlank, input: &mut ButtonController, sfx: &mut Sfx) {
    loop {
        sfx.frame();
        input.update();
        if input.is_just_pressed(agb::input::Button::all()) {
            break;
        }
        vblank.wait_for_vblank();
    }
}

pub fn main(mut gba: agb::Gba) -> ! {
    let objects = gba.display.object.get_managed();
    let vblank = agb::interrupt::VBlank::get();

    let mut rng = RandomNumberGenerator::new();

    let (tiled, mut vram) = gba.display.video.tiled1();
    let mut background1 = tiled.regular(
        Priority::P1,
        display::tiled::RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut background2 = tiled.regular(
        Priority::P0,
        display::tiled::RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut background = background::Background::new(
        &mut background1,
        &mut background2,
        background::Mode::SPLASH,
        &mut vram,
        &mut rng,
    );
    background.commit(&mut vram);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();
    let mut sfx = sfx::Sfx::new(&mut mixer);

    let mut input = agb::input::ButtonController::new();

    loop {
        background.set_mode(background::Mode::SPLASH, &mut vram);
        background.commit(&mut vram);
        objects.commit();

        wait_for_input(&vblank, &mut input, &mut sfx);

        background.set_mode(background::Mode::GAME, &mut vram);
        background.commit(&mut vram);

        {
            let mut apple = apple::Apple::new(&objects, &mut rng);
            let mut snake = snake::Snake::new(3, &objects);
            while snake.is_alive {
                let mut next_input = None;
                for _n_frames in 0..15 {
                    sfx.frame();

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

                snake.try_move(&objects, &mut apple, &mut rng);
                objects.commit();
            }

            wait_for_input(&vblank, &mut input, &mut sfx);
        }
    }
}
