use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::sync::{Arc, Mutex};

mod turntable;
use crate::turntable::Turntable;

mod midi_turntable_controller;
use crate::midi_turntable_controller::new_connection;

mod audio_turntable_controller;
use crate::audio_turntable_controller::AudioTurntableController;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let turntable: Arc<Mutex<Turntable>> = Arc::new(Mutex::new(Turntable::new()));

    // _midi_controller needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _midi_controller = new_connection(&turntable);

    let mut audio_controller = AudioTurntableController::new();

    // Enable raw mode to receive input events
    enable_raw_mode()?;

    loop {
        // Poll for events
        turntable.lock().unwrap().tick();

        audio_controller.set_sound_speed(turntable.lock().unwrap().speed());

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
