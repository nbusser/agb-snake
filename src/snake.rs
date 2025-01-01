use agb::{
    display::object::{OamManaged, Object, Sprite},
    fixnum::Vector2D,
    input::Button,
    rng::RandomNumberGenerator,
};
use alloc::vec::Vec;

use crate::{apple::Apple, constants};

static SPRITES: &agb::display::object::Graphics = agb::include_aseprite!("gfx/snake.aseprite");

static SPRITE_HEAD: &Sprite = SPRITES.tags().get("head").sprite(0);
static SPRITE_HEAD_DEAD: &Sprite = SPRITES.tags().get("head-dead").sprite(0);
static SPRITE_BODY: &Sprite = SPRITES.tags().get("body").sprite(0);

#[derive(PartialEq)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

struct SnakeBodyCell<'a> {
    position: Vector2D<i32>,
    sprite: Object<'a>,
}

pub struct Snake<'a> {
    body: Vec<SnakeBodyCell<'a>>,
    direction: Direction,
    pub is_alive: bool,
}

impl<'a> Snake<'a> {
    fn grow(&mut self, objects: &'a OamManaged<'a>) {
        let tail = &self.body[self.body.len() - 1].position;
        let before_tail = &self.body[self.body.len() - 2].position;

        let mut object = objects.object_sprite(&SPRITE_BODY);
        object.show();

        let offset = *tail - *before_tail;
        self.body.push(SnakeBodyCell {
            position: *tail + offset,
            sprite: object,
        });
    }

    pub fn new(length: u32, objects: &'a OamManaged<'a>) -> Self {
        // Init snake
        let mut snake_starting_body: Vec<SnakeBodyCell> = Vec::new();
        for i in 0..length {
            let sprite = if i == 0 { SPRITE_HEAD } else { SPRITE_BODY };

            let mut object = objects.object_sprite(sprite);
            object.show();

            snake_starting_body.push(SnakeBodyCell {
                position: Vector2D {
                    x: 5 - (i as i32),
                    y: 5,
                },
                sprite: object,
            });
        }
        Self {
            body: snake_starting_body,
            direction: Direction::RIGHT,
            is_alive: true,
        }
    }

    pub fn apply_input(&mut self, input: Button) {
        if let Some(new_direction) = match input {
            Button::UP if self.direction != Direction::DOWN => Some(Direction::UP),
            Button::DOWN if self.direction != Direction::UP => Some(Direction::DOWN),
            Button::LEFT if self.direction != Direction::RIGHT => Some(Direction::LEFT),
            Button::RIGHT if self.direction != Direction::LEFT => Some(Direction::RIGHT),
            _ => None,
        } {
            self.direction = new_direction;
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

    fn move_sprites(&mut self) {
        self.body.iter_mut().for_each(|body_cell| {
            body_cell.sprite.set_position(Vector2D::<i32> {
                x: body_cell.position.x * constants::OBJECTS_SIZE,
                y: body_cell.position.y * constants::OBJECTS_SIZE,
            });
        });
    }

    pub fn try_move(
        &mut self,
        objects: &'a OamManaged<'a>,
        apple: &mut Apple,
        rng: &mut RandomNumberGenerator,
    ) -> bool {
        let head_projection = self.head().position + Snake::get_movement_offset(&self.direction);

        if head_projection.x < constants::BOARD_MIN_X
            || head_projection.x > constants::BOARD_MAX_X
            || head_projection.y < constants::BOARD_MIN_Y
            || head_projection.y > constants::BOARD_MAX_Y
            || self
                .body
                .iter()
                .any(|body_cell| body_cell.position == head_projection)
        {
            self.die(objects);
            return false;
        }

        if head_projection.x == apple.position.x as i32
            && head_projection.y == apple.position.y as i32
        {
            apple.move_apple(rng);
            self.grow(objects);
        }

        for i in (1..self.body.len()).rev() {
            self.body[i].position = self.body[i - 1].position;
        }
        self.body[0].position = head_projection;

        self.move_sprites();

        return true;
    }
}
