use midly::{
    num::{u24, u7}, MetaMessage, MidiMessage, Smf, Track, TrackEvent, TrackEventKind
};
use rizlium_chart::{
    chart::{
        beat2real, Canvas, Chart, ColorRGBA, EasingId, KeyPoint, Line, LinePointData, Note, NoteKind, Spline, ThemeColor, ThemeData
    },
    VIEW_RECT,
};
pub const PIANO_KEY_COUNT: u8 = 88;
pub const C4_POS: u8 = 60;
/// assume that the song is 4/4.
pub(crate) fn tempo2bpm(tempo: u24) -> f32 {
    60. * 1e6 / tempo.as_int() as f32 * 4. / 4.
}

pub(crate) fn key_to_x_value(key: u7) -> f32 {
    // C4(the middle c) is on center.
    (key.as_int() as i8 - C4_POS as i8) as f32 / (PIANO_KEY_COUNT as f32)
        * (VIEW_RECT[1][0] - VIEW_RECT[0][0])
}

pub(crate) fn events_to_bpm<'a>(
    track: impl Iterator<Item = &'a TrackEvent<'a>>,
    ticks_per_beat: u32,
) -> Spline<f32> {
    let mut accumulated_time = 0;
    let beat_bpm: Spline<_> = track
        .filter_map(|a| {
            accumulated_time += a.delta.as_int();
            match a.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(t)) => Some(KeyPoint {
                    time: tick_to_beat(accumulated_time, ticks_per_beat),
                    value: tempo2bpm(t),
                    ease_type: EasingId::Start,
                    relevant: (),
                }),
                _ => None,
            }
        })
        .collect();
    beat_bpm_to_time_bpm(&beat_bpm)
}
fn beat_bpm_to_time_bpm(spline: &Spline<f32>) -> Spline<f32> {
    if spline.points().is_empty() {
        return Spline::EMPTY;
    }
    let mut result = Vec::with_capacity(spline.points().len());
    let mut real_time = 0.0f32;
    let mut last_beat = spline.points()[0].time;
    let mut last_bpm = spline.points()[0].value;

    // 第一个点
    result.push(KeyPoint {
        time: 0.0,
        value: last_bpm,
        ease_type: EasingId::Linear,
        relevant: (),
    });

    for point in &spline.points()[1..] {
        let beat_delta = point.time - last_beat;
        // bpm = 拍/分钟, 1 拍 = 60/bpm 秒
        let time_delta = if last_bpm > 0.0 {
            beat_delta * 60.0 / last_bpm
        } else {
            0.0
        };
        real_time += time_delta;
        result.push(KeyPoint {
            time: real_time,
            value: point.value,
            ease_type: EasingId::Start,
            relevant: (),
        });
        last_beat = point.time;
        last_bpm = point.value;
    }
    Spline::from_iter(result)
}
pub fn midi_track_to_line(track: &Track, line_x: f32, ticks_per_beat: u32) -> Option<Line> {
    let mut current_tick = 0;
    let notes: Vec<_> = track
        .iter()
        .flat_map(|ev| midi_event_to_note(ev, &mut current_tick, ticks_per_beat))
        .collect();
    let start_time = notes.first()?.time;
    let end_time = notes.last()?.time;
    let create_point = |time: f32| -> KeyPoint<f32, LinePointData> {
        KeyPoint {
            time,
            value: line_x,
            ease_type: EasingId::Start,
            relevant: LinePointData {
                canvas: 0,
                color: ColorRGBA::BLACK,
            },
        }
    };
    Some(Line {
        points: Spline::from_iter(vec![create_point(start_time), create_point(end_time)]),
        notes,
        ring_color: Spline::EMPTY,
        line_color: Spline::EMPTY,
    })
}
pub fn midi_track_to_lines(
    track: &Track,
    ticks_per_beat: u32,
) -> Vec<Line> {
    use std::collections::HashMap;
    let mut current_tick = 0u32;
    let mut active_notes: HashMap<u8, (u32, u7)> = HashMap::new();
    let mut lines = Vec::new();

    for event in track.iter() {
        current_tick += event.delta.as_int();
        match event.kind {
            TrackEventKind::Midi {
                channel: _,
                message: MidiMessage::NoteOn { key, vel },
            } if vel.as_int() > 0 => {
                // NoteOn，记录起始tick
                active_notes.insert(key.as_int(), (current_tick, key));
            }
            m@ TrackEventKind::Midi {
                channel: _,
                message: MidiMessage::NoteOff { key, vel },
            }
            |m@  TrackEventKind::Midi {
                channel: _,
                message: MidiMessage::NoteOn { key, vel },
            } => if matches!(m, TrackEventKind::Midi { message: MidiMessage::NoteOff { .. },.. }) || vel == 0{
                // NoteOff 或 NoteOn(vel=0)，结束一条线
                if let Some((start_tick, note_key)) = active_notes.remove(&key.as_int()) {
                    let start_beat = tick_to_beat(start_tick, ticks_per_beat);
                    let end_beat = tick_to_beat(current_tick, ticks_per_beat);
                    let x = key_to_x_value(note_key);

                    let create_point = |time: f32| KeyPoint {
                        time,
                        value: x,
                        ease_type: EasingId::Start,
                        relevant: LinePointData {
                            canvas: 0,
                            color: ColorRGBA::BLACK,
                        },
                    };

                    lines.push(Line {
                        points: Spline::from_iter(vec![
                            create_point(start_beat),
                            create_point(end_beat),
                        ]),
                        notes: Vec::new(),
                        ring_color: Spline::from_iter(vec![
                            KeyPoint {
                                time:0.0,
                                value: ColorRGBA::BLACK,
                                ease_type:EasingId::Start,
                                relevant:()
                            }

                        ]),
                        line_color: Spline::EMPTY,
                    });
                }
            }
            _ => {}
        }
    }
    lines
}

pub fn midi_event_to_note(
    event: &TrackEvent,
    current_tick: &mut u32,
    ticks_per_beat: u32,
) -> std::option::Option<rizlium_chart::chart::Note> {
    *current_tick += dbg!(event.delta.as_int());
    match event.kind {
        midly::TrackEventKind::Midi {
            message: MidiMessage::NoteOn { .. },
            ..
        } => Some(Note::new(
            tick_to_beat(*current_tick, ticks_per_beat),
            NoteKind::Tap,
        )),
        _ => None,
    }
}

pub fn tick_to_beat(tick: u32, ticks_per_beat: u32) -> f32 {
    assert!(ticks_per_beat != 0);
    tick as f32 / ticks_per_beat as f32
}


pub fn build_chart(smf: &Smf, ticks_per_beat: u32) -> rizlium_chart::chart::Chart {
    Chart {
        themes: vec![ThemeData {
            color: ThemeColor {
                background: ColorRGBA::WHITE,
                note: ColorRGBA::WHITE,
                fx: ColorRGBA::WHITE,
            },
            is_challenge: false,
        }],
        theme_control: Spline::from_iter(vec![KeyPoint {
            time: 0.0,
            value: 0,
            ease_type: EasingId::Start,
            relevant: (),
        }]),
        bpm: events_to_bpm(smf.tracks.iter().flatten(), ticks_per_beat),
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
                value: 1000.0,
                ease_type: EasingId::QuadOut, // TODO: this is a bug
                relevant: (),
            }]),
            
        }],
        lines: smf.tracks.iter().flat_map(|t| midi_track_to_lines(t, ticks_per_beat)).collect()
    }
}