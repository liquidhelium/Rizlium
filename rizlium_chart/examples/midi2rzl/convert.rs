use midly::{
    num::{u24, u7},
    MetaMessage, MidiMessage, Track, TrackEvent, TrackEventKind,
};
use rizlium_chart::{
    chart::{
        beat2real, ColorRGBA, EasingId, KeyPoint, Line, LinePointData, Note, NoteKind, Spline,
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
