use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct GameSettings {
    pub music: usize, // volume of music, 0-8
    pub sound: usize, // volumn of sound, 0-8
}

#[derive(Component, Clone, Copy)]
pub enum GameSettingsType {
    Music,
    Sound,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl GameSettings {
    pub const DEFAULT: Self = Self { music: 2, sound: 2 };
    pub const MUSIC_MAX: usize = 8;
    pub const SOUND_MAX: usize = 8;
}
