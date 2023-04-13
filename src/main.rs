use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    tween::Tween,
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{stdin, stdout, Write};

use std::time::Duration;

use midir::{Ignore, MidiInput};

use std::sync::{Arc, Mutex};

use midly::{live::LiveEvent, MidiMessage};

use num_traits::Float;

mod turntable;
use turntable::Turntable;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let turntable: Arc<Mutex<Turntable>> = Arc::new(Mutex::new(Turntable::new()));

    let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
    let filename = "resources/Bazz.wav";
    println!("Loading... {}", &filename);
    let sound_data = StaticSoundData::from_file(filename, StaticSoundSettings::new()).unwrap();
    let mut sound = manager.play(sound_data.clone()).unwrap();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_, message, turntable| {
            on_midi(message, &mut *turntable.lock().unwrap());
        },
        turntable.clone(),
    )?;

    println!(
        "Connection open, reading input from '{}' (press q or Esc to exit) ...",
        in_port_name
    );

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

fn on_midi(event: &[u8], turntable: &mut Turntable) {
    let event = LiveEvent::parse(event).unwrap();
    match event {
        LiveEvent::Midi { channel, message } => match message {
            MidiMessage::NoteOn { key, vel } => {
                if key == 54 {
                    if vel == 0 {
                        turntable.release_vinyl();
                    }

                    if vel == 127 {
                        turntable.catch_vinyl();
                    }
                }
            }
            MidiMessage::Controller { controller, value } => {
                if channel == 0 {
                    if controller == 0 {
                        turntable.tempo =
                            map_range(value.as_int() as f64, 0.0, i8::MAX as f64, 0.92, 1.08);
                    }
                    if controller == 32 {
                        turntable.tempo +=
                            map_range(value.as_int() as f64, 0.0, i8::MAX as f64, 0.000, 0.001);
                    }

                    if controller == 34 {
                        if value == 65 {
                            turntable.impulse_vinyl_clockwise();
                        }
                        if value == 63 {
                            turntable.impulse_vinyl_counterclockwise();
                        }
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
}

fn map_range<T: Float>(value: T, from_low: T, from_high: T, to_low: T, to_high: T) -> T {
    (value - from_low) * (to_high - to_low) / (from_high - from_low) + to_low
}

fn update_sound_speed(sound: &mut StaticSoundHandle, new_speed: f64) {
    sound
        .set_playback_rate(
            new_speed,
            Tween {
                duration: Duration::from_millis(10),
                ..Default::default()
            },
        )
        .unwrap();
}
