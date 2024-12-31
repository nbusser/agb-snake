#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use agb::{
    display::{
        self,
        object::{OamManaged, Object, Sprite},
        tiled::{RegularMap, TileFormat, TileSetting, TiledMap, VRamManager},
        Priority,
    },
    fixnum::Vector2D,
    include_background_gfx, println,
    sound::mixer::Frequency,
};

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
static SPRITE_HEAD_DEAD: &Sprite = SPRITES.tags().get("head-dead").sprite(0);
static SPRITE_BODY: &Sprite = SPRITES.tags().get("body").sprite(0);

#[derive(PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

struct SnakeBodyCell<'a> {
    position: Vector2D<i32>,
    sprite: Object<'a>,
}

struct Snake<'a> {
    body: Vec<SnakeBodyCell<'a>>,
    direction: Direction,
    is_alive: bool,
}

impl Snake<'_> {
    const SNAKE_TILE_SIZE: i32 = 16;
    const MIN_X: i32 = 0;
    const MAX_X: i32 = (display::WIDTH / Snake::SNAKE_TILE_SIZE) - 1;
    const MIN_Y: i32 = 0;
    const MAX_Y: i32 = (display::HEIGHT / Snake::SNAKE_TILE_SIZE) - 1;

    pub fn display(&mut self, objects: &OamManaged) {
        self.body.iter_mut().for_each(|body_cell| {
            body_cell.sprite.set_position(Vector2D::<i32> {
                x: body_cell.position.x * Snake::SNAKE_TILE_SIZE,
                y: body_cell.position.y * Snake::SNAKE_TILE_SIZE,
            });
            body_cell.sprite.show();
        });
        objects.commit();
    }

    pub fn apply_input(&mut self, input: Direction) {
        if match input {
            Direction::UP => self.direction != Direction::DOWN,
            Direction::DOWN => self.direction != Direction::UP,
            Direction::LEFT => self.direction != Direction::RIGHT,
            Direction::RIGHT => self.direction != Direction::LEFT,
        } {
            self.direction = input;
        }
    }

    fn get_movement_offset(direction: &Direction) -> Vector2D<i32> {
        match direction {
            Direction::UP => Vector2D::<i32> { x: 0, y: -1 },
            Direction::DOWN => Vector2D::<i32> { x: 0, y: 1 },
            Direction::LEFT => Vector2D::<i32> { x: -1, y: 0 },
            Direction::RIGHT => Vector2D::<i32> { x: 1, y: 0 },
        }
    }

    fn head(&self) -> &SnakeBodyCell<'_> {
        &self.body[0]
    }

    fn die(&mut self, objects: &OamManaged) {
        self.is_alive = false;
        self.body[0]
            .sprite
            .set_sprite(objects.sprite(&SPRITE_HEAD_DEAD));
    }

    pub fn try_move(&mut self, objects: &OamManaged) -> bool {
        let head_projection = self.head().position + Snake::get_movement_offset(&self.direction);

        if head_projection.x < Snake::MIN_X
            || head_projection.x > Snake::MAX_X
            || head_projection.y < Snake::MIN_Y
            || head_projection.y > Snake::MAX_Y
            || self
                .body
                .iter()
                .any(|body_cell| body_cell.position == head_projection)
        {
            self.die(objects);
            return false;
        }

        for i in (1..self.body.len()).rev() {
            self.body[i].position = self.body[i - 1].position;
        }

        self.body[0].position = head_projection;
        return true;
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
    for i in 0..5 {
        let sprite = if i == 0 { SPRITE_HEAD } else { SPRITE_BODY };

        snake_starting_body.push(SnakeBodyCell {
            position: Vector2D { x: 5 - i, y: 5 },
            sprite: objects.object_sprite(sprite),
        });
    }
    let mut snake = Snake {
        body: snake_starting_body,
        direction: Direction::RIGHT,
        is_alive: true,
    };

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
                        snake.apply_input(Direction::UP);
                    } else if input.is_just_pressed(agb::input::Button::DOWN) {
                        snake.apply_input(Direction::DOWN);
                    } else if input.is_just_pressed(agb::input::Button::LEFT) {
                        snake.apply_input(Direction::LEFT);
                    } else if input.is_just_pressed(agb::input::Button::RIGHT) {
                        snake.apply_input(Direction::RIGHT);
                    }
                    vblank.wait_for_vblank();
                }
                snake.try_move(&objects);
                snake.display(&objects);
            }
        }
    }
}
