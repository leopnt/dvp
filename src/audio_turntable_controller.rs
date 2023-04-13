use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    tween::Tween,
};

use std::time::Duration;

pub struct AudioTurntableController {
    _manager: AudioManager<CpalBackend>,
    _current_file_name: String,
    _sound_data: StaticSoundData,
    sound: StaticSoundHandle,
}

impl AudioTurntableController {
    pub fn new() -> Self {
        let mut manager =
            AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        let current_file_name = String::from("resources/Bazz.wav");
        println!("Loading... {}", &current_file_name);
        let sound_data =
            StaticSoundData::from_file(&current_file_name, StaticSoundSettings::new()).unwrap();
        let sound = manager.play(sound_data.clone()).unwrap();

        AudioTurntableController {
            _manager: manager,
            _current_file_name: current_file_name,
            _sound_data: sound_data,
            sound: sound,
        }
    }

    pub fn set_sound_speed(&mut self, new_speed: f64) {
        self.sound
            .set_playback_rate(
                new_speed,
                Tween {
                    duration: Duration::from_millis(0),
                    ..Default::default()
                },
            )
            .unwrap();
    }

    pub fn get_position(&self) -> f64 {
        self.sound.position()
    }

    pub fn set_position(&mut self, position: f64) {
        self.sound.seek_to(position).unwrap();
    }
}
