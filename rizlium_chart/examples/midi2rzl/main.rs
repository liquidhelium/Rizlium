//! Convert MIDI files to rizlium charts.

use std::{error, fs::{self, File}};

use midly::{Smf, Timing};
use rizlium_chart::chart::{
    Canvas, Chart, ColorRGBA, EasingId, KeyPoint, Spline, ThemeColor, ThemeData,
};
use serde_json::to_writer_pretty;

use crate::convert::midi_track_to_line;

mod convert;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Box<dyn error::Error>> {
    let file =
        fs::read("/home/helium/code/Rizlium/rizlium_chart/examples/midi2rzl/assets/abyss.mid")?;
    let smf = Smf::parse(&file)?;
    let ticks_per_beat = match smf.header.timing {
        Timing::Metrical(t) => t.as_int() as u32,
        _ => Err(midly::Error::new(&midly::ErrorKind::Invalid(
            "We support tick per beat times only.",
        )))?,
    };
    let chart = Chart {
        themes: vec![ThemeData {
            color: ThemeColor {
                background: ColorRGBA::WHITE,
                note: ColorRGBA::WHITE,
            },
            is_challenge: false,
        }],
        theme_control: Spline::from_iter(vec![KeyPoint {
            time: 0.0,
            value: 0,
            ease_type: EasingId::Start,
            relevant: (),
        }]),
        bpm: convert::events_to_bpm(smf.tracks.iter().flatten(), ticks_per_beat),
        cam_move: Spline::from_iter(vec![KeyPoint {
            time: 0.0,
            value: 0.,
            ease_type: EasingId::Start,
            relevant: (),
        }]),
        cam_scale: Spline::from_iter(vec![KeyPoint {
            time: 0.0,
            value: 1.0,
            ease_type: EasingId::Start,
            relevant: (),
        }]),
        canvases: vec![Canvas {
            x_pos: Spline::from_iter(vec![KeyPoint {
                time: 0.0,
                value: 0.0,
                ease_type: EasingId::Start,
                relevant: (),
            }]),
            speed: Spline::from_iter(vec![KeyPoint {
                time: 0.0,
                value: 1.0,
                ease_type: EasingId::Start,
                relevant: (),
            }]),
            
        }],
        lines: smf.tracks.iter().flat_map(|t| midi_track_to_line(t, 0.0,ticks_per_beat)).collect()
    };
    let file = File::create("1.json")?;
    to_writer_pretty(file, &chart)?;

    Ok(())
}
