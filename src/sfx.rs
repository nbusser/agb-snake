use agb::{
    include_wav,
    sound::mixer::{Mixer, SoundChannel},
};

static BGM: &[u8] = include_wav!("sfx/bgm.wav");

pub struct Sfx<'a> {
    mixer: &'a mut Mixer<'a>,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer<'a>) -> Self {
        let mut title_music = SoundChannel::new_high_priority(BGM);
        title_music.should_loop();
        mixer.play_sound(title_music).unwrap();

        Self { mixer }
    }

    pub fn frame(&mut self) {
        self.mixer.frame();
    }
}
