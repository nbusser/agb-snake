use agb::{
    include_wav,
    sound::{
        self,
        dmg::Sound,
        mixer::{Mixer, SoundChannel},
    },
};

static BGM: &[u8] = include_wav!("sfx/bgm.wav");

pub struct Sfx<'a> {
    mixer: &'a mut Mixer<'a>,
    dmg: &'a mut Sound,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer<'a>, dmg: &'a mut Sound) -> Self {
        let mut title_music = SoundChannel::new_high_priority(BGM);
        title_music.should_loop();
        mixer.play_sound(title_music).unwrap();

        Self { mixer, dmg }
    }

    pub fn frame(&mut self) {
        self.mixer.frame();
    }

    pub fn play_eat_apple(&mut self) {
        let sweep_settings = sound::dmg::SweepSettings::default();
        self.dmg.channel1().play_sound(
            1024,
            Some(5),
            &sweep_settings,
            &sound::dmg::EnvelopeSettings::default(),
            sound::dmg::DutyCycle::ThreeQuarters,
        );
        self.dmg.channel2().play_sound(
            1524,
            Some(10),
            &sound::dmg::EnvelopeSettings::default(),
            sound::dmg::DutyCycle::Half,
        );
    }

    pub fn play_death_sound(&mut self) {
        self.dmg.noise().play_sound(
            Some(16),
            &sound::dmg::EnvelopeSettings::default(),
            2,
            false,
            1,
        );
    }
}
