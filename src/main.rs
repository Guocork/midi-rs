use eframe::egui;
use itertools::Itertools;
use phf::{phf_map, Map};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::{
    fs::File,
    sync::{Arc, Mutex},
};
use tinyaudio::prelude::*;

const OUTPUT_PARAMS: OutputDeviceParameters = OutputDeviceParameters {
    channels_count: 2,
    sample_rate: 44100,
    channel_sample_count: 441, // 样本的最大长度
};


#[derive(Debug)]
pub struct MidiNote {
    pub note: i32,
    pub velocity: i32,
}

pub static NOTE_KEY_MAP: Map<&'static str, MidiNote> = phf_map! {
    "A" => MidiNote {
        note: 60,
        velocity: 100,
    },
    "S" => MidiNote {
        note: 62,
        velocity: 100,
    },
    "D" => MidiNote {
        note: 64,
        velocity: 100,
    },
    "F" => MidiNote {
        note: 65,
        velocity: 100,
    },
    "G" => MidiNote {
        note: 67,
        velocity: 100,
    },
};

struct SynthApp {
    synthesizer: Arc<Mutex<Synthesizer>>,
    midi_channel: i32,
}

impl SynthApp {
    fn note_on(&mut self, key: &str) {
        let note = match NOTE_KEY_MAP.get(key) {
            Some(note) => note,
            None => return,
        };
        self.synthesizer
            .lock()
            .unwrap()
            .note_on(self.midi_channel, note.note, note.velocity)
    }

    fn note_off(&mut self, key: &str) {
        let note = match NOTE_KEY_MAP.get(key) {
            Some(note) => note,
            None => return,
        };
        self.synthesizer
            .lock()
            .unwrap()
            .note_off(self.midi_channel, note.note);
    }
}


impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            for key_str in NOTE_KEY_MAP.keys() {
                if let Some(key) = egui::Key::from_name(key_str) {
                    if i.key_pressed(key) {
                        self.note_on(key_str);
                    } else if i.key_released(key) {
                        self.note_off(key_str);
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.label(format!("Midi channel {}", self.midi_channel));
        });
    }
}
fn main() -> Result<(), eframe::Error> {
    // 加载音色库
    let mut sf2 = File::open("TimGM6mb.sf2").unwrap();
    let sound_font = Arc::new(SoundFont::new(&mut sf2).unwrap());

    // 创建MIDI文件序列器
    let settings = SynthesizerSettings::new(OUTPUT_PARAMS.sample_rate as i32);
    let synthesizer = Arc::new(Mutex::new(
        Synthesizer::new(&sound_font, &settings).unwrap(),
    ));

    // 运行输出设备
    let synth_c = synthesizer.clone();
    let mut left: Vec<f32> = vec![0_f32; OUTPUT_PARAMS.channel_sample_count];
    let mut right: Vec<f32> = vec![0_f32; OUTPUT_PARAMS.channel_sample_count];
    let _device = run_output_device(OUTPUT_PARAMS, move |data| {
        synth_c
            .lock()
            .unwrap()
            .render(&mut left[..], &mut right[..]);
        for (i, value) in left.iter().interleave(right.iter()).enumerate() {
            data[i] = *value;
        }
    })
    .unwrap();

    // eframe
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| {
            Box::new(SynthApp {
                synthesizer,
                midi_channel: 0,
            })
        }),
    )
}
