use lerp::Lerp;
use std::time::Instant;

use crate::audio_turntable_controller::AudioTurntableController;

pub struct Turntable {
    pub tempo: f64,
    vinyl_speed: f64,
    vinyl_lock: bool,
    speed: f64,
    torque: f64, // how fast the speed reached the vinyl speed
    impulses_per_rotation: usize,
    cumulative_impulse: f64,
    last_tick_timestamp: Instant,
    play: bool,
    audio_controller: AudioTurntableController,
    cue: f64,
}

impl Turntable {
    pub fn new() -> Self {
        Turntable {
            vinyl_speed: 1.0,
            vinyl_lock: false,
            tempo: 1.0,
            speed: 1.0,
            torque: 0.3,
            impulses_per_rotation: 500, // number of MIDI events to make one rotation
            cumulative_impulse: 0.0,
            last_tick_timestamp: Instant::now(),
            play: true,
            audio_controller: AudioTurntableController::new(),
            cue: 0.0,
        }
    }

    pub fn catch_vinyl(&mut self) {
        self.vinyl_speed = 0.0;
        self.vinyl_lock = true;
    }

    pub fn release_vinyl(&mut self) {
        self.vinyl_speed = self.tempo;
        self.vinyl_lock = false;
    }

    pub fn impulse_vinyl_clockwise(&mut self) {
        self.cumulative_impulse += 1.0;
    }

    pub fn impulse_vinyl_counterclockwise(&mut self) {
        self.cumulative_impulse -= 1.0;
    }

    pub fn toggle_play(&mut self) {
        self.play = !self.play;
    }

    pub fn cue(&mut self) -> f64 {
        if self.vinyl_lock {
            self.cue = self.audio_controller.get_position();
            return self.cue;
        }

        self.audio_controller.set_position(self.cue);
        return self.cue;
    }

    pub fn tick(&mut self) {
        let dt = self.last_tick_timestamp.elapsed();
        let dist = self.cumulative_impulse / self.impulses_per_rotation as f64;
        let dv = dist / dt.as_secs_f64();

        self.vinyl_speed = dv;

        if self.vinyl_lock {
            self.speed = self.vinyl_speed;
        } else {
            if self.play {
                self.speed = self.speed.lerp(self.tempo, self.torque);
            } else {
                self.speed = self.speed.lerp(0.0, self.torque);
            }
        }

        self.cumulative_impulse = 0.0;
        self.last_tick_timestamp = Instant::now();

        self.audio_controller.set_sound_speed(self.speed);
    }
}
