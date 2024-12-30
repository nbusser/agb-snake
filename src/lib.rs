#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use core::borrow::Borrow;

use agb::{
    display::{
        self,
        object::{OamManaged, Object, Sprite},
        tiled::{RegularMap, TileFormat, TileSetting, TiledMap, VRamManager},
        Priority,
    },
    fixnum::Vector2D,
    include_background_gfx,
    sound::mixer::Frequency,
};

use alloc::vec;
use alloc::vec::Vec;

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

static SPRITES: &agb::display::object::Graphics = agb::include_aseprite!("gfx/snake.aseprite");

static SPRITE_HEAD: &Sprite = SPRITES.tags().get("head").sprite(0);
static SPRITE_BODY: &Sprite = SPRITES.tags().get("body").sprite(0);

type Position = Vector2D<u16>;

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

struct SnakeBodyCell<'a> {
    position: Position,
    sprite: Object<'a>,
}

const SNAKE_TILE_SIZE: u16 = 16;

struct Snake<'a> {
    body: Vec<SnakeBodyCell<'a>>,
    direction: Direction,
}

impl Snake<'_> {
    pub fn display(&mut self, objects: &OamManaged) {
        self.body.iter_mut().for_each(|body_cell| {
            body_cell.sprite.set_position(Vector2D::<i32> {
                x: (body_cell.position.x * SNAKE_TILE_SIZE) as i32,
                y: (body_cell.position.y * SNAKE_TILE_SIZE) as i32,
            });
            body_cell.sprite.show();
        });
        objects.commit();
    }
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

    // Init snake
    let mut snake_starting_body: Vec<SnakeBodyCell> = Vec::new();
    for i in 0..3 {
        let sprite = if i == 0 { SPRITE_HEAD } else { SPRITE_BODY };

        snake_starting_body.push(SnakeBodyCell {
            position: Vector2D { x: 5 - i, y: 5 },
            sprite: objects.object_sprite(sprite),
        });
    }
    let mut snake = Snake {
        body: snake_starting_body,
        direction: Direction::RIGHT,
    };

    loop {
        create_background(&mut background, &mut vram);
        create_grid(&mut background_grid, &mut vram);

        loop {
            snake.display(&objects);
        }
    }
}
