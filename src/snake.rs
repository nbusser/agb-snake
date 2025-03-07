use agb::{
    display::object::{OamManaged, Object, Sprite, Tag},
    fixnum::Vector2D,
    input::Button,
    rng::RandomNumberGenerator,
};
use alloc::vec::Vec;

use crate::{apple::Apple, constants, sfx::Sfx};

static SPRITES: &agb::display::object::Graphics = agb::include_aseprite!("gfx/snake.aseprite");

static SPRITE_HEAD_TAG: &Tag = SPRITES.tags().get("head");

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
    animation_frame: usize,
    pub is_alive: bool,
}

impl<'a> Snake<'a> {
    pub fn length(&self) -> usize {
        self.body.len()
    }

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
            let sprite = if i == 0 {
                SPRITE_HEAD_TAG.sprite(0)
            } else {
                SPRITE_BODY
            };

            let object_position = Vector2D {
                x: 5 - (i as i32),
                y: 5,
            };

            let mut object = objects.object_sprite(sprite);
            object.set_position(object_position * constants::OBJECTS_SIZE);
            object.show();

            let body_cell = SnakeBodyCell {
                position: object_position,
                sprite: object,
            };

            snake_starting_body.push(body_cell);
        }
        Self {
            body: snake_starting_body,
            direction: Direction::RIGHT,
            animation_frame: 0,
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

    fn die(&mut self, objects: &OamManaged, sfx: &mut Sfx) {
        self.is_alive = false;
        self.body[0]
            .sprite
            .set_sprite(objects.sprite(&SPRITE_HEAD_DEAD));
        sfx.play_death_sound();
    }

    fn move_sprites(&mut self) {
        self.body.iter_mut().for_each(|body_cell| {
            body_cell.sprite.set_position(Vector2D::<i32> {
                x: body_cell.position.x * constants::OBJECTS_SIZE,
                y: body_cell.position.y * constants::OBJECTS_SIZE,
            });
        });
    }

    const ANIMATION_SLOWDOWN_FACTOR: usize = 10;

    pub fn frame_anim(&mut self, objects: &'a OamManaged<'a>) -> bool {
        self.animation_frame = (self.animation_frame + 1)
            % (SPRITE_HEAD_TAG.sprites().len() * Self::ANIMATION_SLOWDOWN_FACTOR);
        self.body[0].sprite.set_sprite(objects.sprite(
            SPRITE_HEAD_TAG.sprite(self.animation_frame / Self::ANIMATION_SLOWDOWN_FACTOR),
        ));
        self.animation_frame
            .rem_euclid(Self::ANIMATION_SLOWDOWN_FACTOR)
            == 0
    }

    pub fn frame(
        &mut self,
        objects: &'a OamManaged<'a>,
        apple: &mut Apple,
        rng: &mut RandomNumberGenerator,
        sfx: &mut Sfx,
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
            self.die(objects, sfx);
            return false;
        }

        if head_projection.x == apple.position.x as i32
            && head_projection.y == apple.position.y as i32
        {
            apple.move_apple(rng);
            sfx.play_eat_apple();
            self.grow(objects);
        }

        for i in (1..self.body.len()).rev() {
            self.body[i].position = self.body[i - 1].position;
        }
        self.body[0].position = head_projection;

        self.move_sprites();

        self.frame_anim(objects);

        return true;
    }
}
