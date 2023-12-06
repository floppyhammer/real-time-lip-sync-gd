use rlip_sync::lip_sync::*;
use std::time::SystemTime;

use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

fn main() {
    let working_dir = std::env::current_dir();
    println!("Working directory: {:?}", working_dir.unwrap());

    // Create an audio manager, which plays sounds and manages resources.
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    let sound_data =
        match StaticSoundData::from_file("test_sound.ogg", StaticSoundSettings::default()) {
            Ok(data) => data,
            Err(error) => {
                println!("Failed to load audio file: {:?}", error);
                panic!();
            }
        };

    let res = manager.play(sound_data.clone());
    if res.is_err() {
        println!("Playing sound failed!");
    }

    let mut lip_sync = LipSync::new();

    let start_time = SystemTime::now();

    let mut last_sec_handled = 0;

    loop {
        match start_time.elapsed() {
            Ok(elapsed) => {
                let current_sec = elapsed.as_secs();
                if current_sec > last_sec_handled {
                    let frame_range = (
                        last_sec_handled * sound_data.sample_rate as u64,
                        current_sec * sound_data.sample_rate as u64,
                    );

                    let mut stream = Vec::new();

                    for frame_index in frame_range.0..frame_range.1 {
                        if frame_index < sound_data.frames.len() as u64 {
                            stream.push(sound_data.frames[frame_index as usize].left);
                        }
                    }

                    lip_sync.update(stream);

                    match lip_sync.poll() {
                        None => {}
                        Some(est) => {
                            println!("{:?} {:?} {:?}", est.estimate, est.vowel, est.amount);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {e:?}");
            }
        }
    }
}
