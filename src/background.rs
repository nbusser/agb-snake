use agb::{
    display::{
        self,
        blend::{Blend, Layer},
        tiled::{BackgroundID, RegularMap, TileSetting, TiledMap, VRamManager},
    },
    fixnum::Num,
    include_background_gfx,
    rng::RandomNumberGenerator,
};

use crate::constants;

include_background_gfx!(backgrounds, "121105",
    background => deduplicate "gfx/plain-background.png",
    props => deduplicate "gfx/props.aseprite",
    splash => deduplicate "gfx/splash.aseprite"
);

pub enum Mode {
    SPLASH,
    GAME,
}

#[derive(PartialEq)]
pub enum FadeDirection {
    FadeIn,
    FadeOut,
}

struct FadeOperation {
    direction: FadeDirection,
    increase_factor: f64,
    current_fade: f64,
}

pub struct Background<'a> {
    background1: &'a mut RegularMap,
    background2: &'a mut RegularMap,
    blend: Blend<'a>,
    fade: Option<FadeOperation>,
}

fn build_props(
    background2: &mut RegularMap,
    vram: &mut VRamManager,
    rng: &mut RandomNumberGenerator,
) {
    const NB_PROPS_TILES: i32 = 3;
    const N_TILES_X: u32 = (display::WIDTH / constants::TILE_SIZE) as u32;
    const N_TILES_Y: u32 = (display::HEIGHT / constants::TILE_SIZE) as u32;
    for x in 0..N_TILES_X {
        for y in 0..N_TILES_Y {
            if rng.gen().rem_euclid(100) < 2 {
                let tile_id = rng.gen().rem_euclid(NB_PROPS_TILES) as u16;
                background2.set_tile(
                    vram,
                    (x as u16, y as u16),
                    &backgrounds::props.tiles,
                    TileSetting::new(tile_id, false, false, 0),
                );
            };
        }
    }
}

pub struct RegularMapAndId<'a> {
    pub id: BackgroundID,
    pub map: &'a mut RegularMap,
}

impl<'a> Background<'a> {
    pub fn set_mode(&mut self, mode: Mode, vram: &mut VRamManager) {
        self.background1.fill_with(
            vram,
            match mode {
                Mode::SPLASH => &backgrounds::splash,
                Mode::GAME => &backgrounds::background,
            },
        );
        self.background2.set_visible(match mode {
            Mode::SPLASH => false,
            Mode::GAME => true,
        });
    }

    pub fn new(
        background1: RegularMapAndId<'a>,
        background2: RegularMapAndId<'a>,
        mut blend: Blend<'a>,
        mode: Mode,
        vram: &mut VRamManager,
        rng: &mut RandomNumberGenerator,
    ) -> Self {
        background1.map.set_scroll_pos((0i16, 0));
        vram.set_background_palettes(backgrounds::PALETTES);
        background1.map.set_visible(true);

        build_props(background2.map, vram, rng);

        blend
            .set_background_enable(Layer::Top, background1.id, true)
            .set_background_enable(Layer::Top, background2.id, true)
            .set_object_enable(Layer::Top, true)
            .set_backdrop_enable(Layer::Bottom, true)
            .set_blend_mode(display::blend::BlendMode::FadeToBlack);

        let mut instance = Self {
            background1: background1.map,
            background2: background2.map,
            blend,
            fade: None,
        };

        instance.set_mode(mode, vram);
        instance
    }

    pub fn commit(&mut self, vram: &mut VRamManager) {
        self.background1.commit(vram);
        self.background2.commit(vram);
    }

    pub fn start_fade(&mut self, direction: FadeDirection) {
        self.blend.reset_fades();
        let start_value = match direction {
            FadeDirection::FadeIn => 1.0,
            FadeDirection::FadeOut => 0.0,
        };
        self.fade = Some(FadeOperation {
            direction,
            increase_factor: 0.05,
            current_fade: start_value,
        });
    }

    pub fn fade_frame(&mut self) -> bool {
        let fade = self.fade.as_mut().unwrap();
        let is_finished = match fade.direction {
            FadeDirection::FadeIn => {
                fade.current_fade -= fade.increase_factor;
                fade.current_fade <= 0.0
            }
            FadeDirection::FadeOut => {
                fade.current_fade += fade.increase_factor;
                fade.current_fade >= 1.0
            }
        };
        self.blend.set_fade(Num::from_f64(fade.current_fade));
        self.blend.commit();
        is_finished
    }
}
