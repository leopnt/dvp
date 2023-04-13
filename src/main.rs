use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    tween::Tween,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::time::Duration;
use std::sync::{Arc, Mutex};

mod turntable;
use crate::turntable::Turntable;

mod midi_turntable_controller;
use crate::midi_turntable_controller::new_connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let turntable: Arc<Mutex<Turntable>> = Arc::new(Mutex::new(Turntable::new()));

    // _midi_controller needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _midi_controller = new_connection(&turntable);

    let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
    let filename = "resources/Bazz.wav";
    println!("Loading... {}", &filename);
    let sound_data = StaticSoundData::from_file(filename, StaticSoundSettings::new()).unwrap();
    let mut sound = manager.play(sound_data.clone()).unwrap();

    // Enable raw mode to receive input events
    enable_raw_mode()?;

    loop {
        // Poll for events
        turntable.lock().unwrap().tick();
        update_sound_speed(&mut sound, turntable.lock().unwrap().speed());

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    // Print out the key code if it's a printable character or a special key
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Clear the screen and exit the loop
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Disable raw mode before exiting
    disable_raw_mode()?;

    Ok(())
}

fn update_sound_speed(sound: &mut StaticSoundHandle, new_speed: f64) {
    sound
        .set_playback_rate(
            new_speed,
            Tween {
                duration: Duration::from_millis(0),
                ..Default::default()
            },
        )
        .unwrap();
}
