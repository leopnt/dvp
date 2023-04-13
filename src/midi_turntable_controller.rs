use midir::{Ignore, MidiInput, MidiInputConnection};
use midly::{live::LiveEvent, MidiMessage};

use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};

use num_traits::Float;

use crate::turntable::Turntable;

pub fn new_connection(
    turntable: &Arc<Mutex<Turntable>>,
) -> Result<MidiInputConnection<Arc<Mutex<Turntable>>>, Box<dyn std::error::Error>> {
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
            let in_port_id = input.trim().parse::<usize>()?;
            in_ports
                .get(in_port_id)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port).unwrap();
    println!(
        "Connection open, reading input from '{}' (press q or Esc to exit) ...",
        in_port_name
    );

    // _conn_in needs to be kept alive
    let _conn_in = midi_in
        .connect(
            in_port,
            "midir-read-input",
            move |_, message, turntable| {
                on_midi(message, &mut *turntable.lock().unwrap());
            },
            turntable.clone(),
        )
        .unwrap();

    Ok(_conn_in)
}

pub fn on_midi(event: &[u8], turntable: &mut Turntable) {
    let event = LiveEvent::parse(event).unwrap();

    // react to hard-coded values for DDJ-400
    match event {
        LiveEvent::Midi { channel, message } => match message {
            MidiMessage::NoteOn { key, vel } => {
                if key == 54 {
                    if vel == 0 {
                        // left jog touch release
                        turntable.release_vinyl();
                    }

                    if vel == 127 {
                        // left jog touch press
                        turntable.catch_vinyl();
                    }
                }

                if key == 11 {
                    if vel == 127 {
                        // left play button press
                        turntable.toggle_play();
                    }
                }

                if key == 12 {
                    if vel == 127 {
                        // left cue button press
                        turntable.cue();
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
