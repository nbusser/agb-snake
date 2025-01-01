use agb::{
    display::{
        self,
        tiled::{RegularMap, TileSetting, TiledMap, VRamManager},
    },
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

pub struct Background<'a> {
    background1: &'a mut RegularMap,
    background2: &'a mut RegularMap,
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
        background1: &'a mut RegularMap,
        background2: &'a mut RegularMap,
        mode: Mode,
        vram: &mut VRamManager,
        rng: &mut RandomNumberGenerator,
    ) -> Self {
        background1.set_scroll_pos((0i16, 0));
        vram.set_background_palettes(backgrounds::PALETTES);
        background1.set_visible(true);

        build_props(background2, vram, rng);

        let mut instance = Self {
            background1,
            background2,
        };

        instance.set_mode(mode, vram);
        instance
    }

    pub fn commit(&mut self, vram: &mut VRamManager) {
        self.background1.commit(vram);
        self.background2.commit(vram);
    }
}
