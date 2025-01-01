use agb::{
    display::object::{OamManaged, Object, Sprite},
    fixnum::Vector2D,
    rng::RandomNumberGenerator,
};

use crate::constants;

static SPRITES: &agb::display::object::Graphics = agb::include_aseprite!("gfx/apple.aseprite");

static SPRITE_APPLE: &Sprite = SPRITES.tags().get("apple").sprite(0);

pub struct Apple<'a> {
    pub position: Vector2D<u16>,
    sprite: Object<'a>,
}

impl<'a> Apple<'a> {
    fn pick_random_pos(&mut self, rng: &mut RandomNumberGenerator) -> Vector2D<u16> {
        let x = rng.gen().abs().rem_euclid(constants::BOARD_MAX_X) as u16;
        let y = rng.gen().abs().rem_euclid(constants::BOARD_MAX_Y) as u16;
        Vector2D { x, y }
    }

    pub fn move_apple(&mut self, rng: &mut RandomNumberGenerator) {
        self.position = self.pick_random_pos(rng);

        self.sprite
            .set_x(self.position.x * constants::OBJECTS_SIZE as u16);
        self.sprite
            .set_y(self.position.y * constants::OBJECTS_SIZE as u16);
    }

    pub fn new(objects: &'a OamManaged<'a>, rng: &mut RandomNumberGenerator) -> Self {
        let mut apple = Self {
            position: Vector2D { x: 0, y: 0 },
            sprite: objects.object_sprite(&SPRITE_APPLE),
        };
        apple.move_apple(rng);
        apple.sprite.show();
        apple
    }
}
