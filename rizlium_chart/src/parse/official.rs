use crate::VIEW_RECT;

use crate::chart::{self, Spline};
use crate::parse::EmptyBPMSnafu;
use chart::ChartCache;
use log::info;
use serde_derive::{Deserialize, Serialize};
use snafu::{ensure, OptionExt};

use super::{ConvertError, ConvertResult, HoldNoEndSnafu};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub colors_list: [ColorRGBA; 3],
}

impl Theme {
    fn convert(self, is_challenge: bool) -> chart::ThemeData {
        // TODO: index 2 unknown
        let [bg, note, _] = self.colors_list;
        chart::ThemeData {
            color: chart::ThemeColor {
                background: bg.into(),
                note: note.into(),
            },
            is_challenge,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeTime {
    pub check_point: f32,

    pub start: f32,

    pub end: f32,

    pub trans_time: f32,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    pub note_type: u8,

    pub time: f32,

    pub floor_position: f32,

    pub other_informations: Vec<f32>,
}
impl Note {
    fn convert(self, line_idx: usize, note_idx: usize) -> ConvertResult<chart::Note> {
        Ok(chart::Note::new(
            self.time,
            match self.note_type {
                0 => chart::NoteKind::Tap,
                1 => chart::NoteKind::Drag,
                2 => chart::NoteKind::Hold {
                    end: *self
                        .other_informations
                        .get(0)
                        .with_context(|| HoldNoEndSnafu { line_idx, note_idx })?,
                },
                otherwise => {
                    return Err(ConvertError::UnknownNoteKind {
                        raw_kind: otherwise as usize,
                    })
                }
            },
        ))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorRGBA {
    pub r: u8,

    pub g: u8,

    pub b: u8,

    pub a: u8,
}

impl Into<chart::ColorRGBA> for ColorRGBA {
    fn into(self) -> chart::ColorRGBA {
        chart::ColorRGBA {
            r: self.r as f32 / 255.0,
            g: self.g as f32 / 255.0,
            b: self.b as f32 / 255.0,
            a: self.a as f32 / 255.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinePoint {
    pub time: f32,

    pub x_position: f32,

    pub color: ColorRGBA,

    pub ease_type: u8,

    pub canvas_index: usize,

    pub floor_position: f32,
}

impl LinePoint {
    fn convert(
        self,
        line_index: usize,
    ) -> ConvertResult<(
        chart::KeyPoint<f32, usize>,
        chart::KeyPoint<chart::ColorRGBA, usize>,
    )> {
        let point = chart::KeyPoint {
            time: self.time,
            value: self.x_position,
            ease_type: self
                .ease_type
                .try_into()
                .or(Err(ConvertError::UnknownEaseKind {
                    raw_kind: self.ease_type,
                }))?,
            relevent: self.canvas_index,
        };
        let color = chart::KeyPoint {
            time: self.time,
            value: self.color.into(),
            ease_type: self
                .ease_type
                .try_into()
                .or(Err(ConvertError::UnknownEaseKind {
                    raw_kind: self.ease_type,
                }))?,
            relevent: line_index,
        };
        Ok((point, color))
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorKeyPoint {
    pub start_color: ColorRGBA,

    pub end_color: ColorRGBA,

    pub time: f32,
}
impl Into<chart::KeyPoint<chart::ColorRGBA>> for ColorKeyPoint {
    fn into(self) -> chart::KeyPoint<chart::ColorRGBA> {
        chart::KeyPoint {
            time: self.time,
            value: self.start_color.into(),
            ease_type: chart::EasingId::Linear,
            relevent: (),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub line_points: Vec<LinePoint>,

    pub notes: Vec<Note>,

    pub judge_ring_color: Vec<ColorKeyPoint>,

    pub line_color: Vec<ColorKeyPoint>,
}
impl Line {
    fn convert(self, line_index: usize) -> ConvertResult<chart::Line> {
        let line_color: Spline<_> = self.line_color.into_iter().map(|k| k.into()).collect();
        let a = self
            .line_points
            .into_iter()
            .map(|p| p.convert(line_index))
            .collect::<ConvertResult<Vec<_>>>()?;
        let (points, colors): (Vec<_>, Vec<_>) = a.into_iter().unzip();
        let points: Spline<_, _> = points
            .into_iter()
            .map(|mut x| {
                x.value = scale_x(x.value);
                x
            })
            .collect();
        let colors = colors.into(); // todo: color mix
        let notes: Vec<chart::Note> = self
            .notes
            .into_iter()
            .enumerate()
            .map(|(idx, n)| n.convert(line_index, idx))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(chart::Line {
            points,
            point_color: colors,
            notes,
            ring_color: self
                .judge_ring_color
                .into_iter()
                .map(|k| k.into())
                .collect(),
            line_color,
        })
    }
}

fn scale_x(x: f32) -> f32 {
    (x + 0.5) * (VIEW_RECT[1][0] - VIEW_RECT[0][0])
}
fn scale_y(y: f32) -> f32 {
    y * (VIEW_RECT[1][1] - VIEW_RECT[0][1]) 
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasMove {
    pub index: i32,

    pub x_position_key_points: Vec<KeyPoint>,

    pub speed_key_points: Vec<KeyPoint>,
}

impl CanvasMove {
    fn convert(self, beat_to_time: &Spline<f32>) -> ConvertResult<chart::Canvas> {
        Ok(chart::Canvas {
            x_pos: self
                .x_position_key_points
                .into_iter()
                .map(|p| p.try_into())
                .map(|p: Result<chart::KeyPoint<f32>, _>| {
                    let mut p = p?;
                    p.value = scale_x(p.value);
                    Ok(p)
                })
                .collect::<Result<_, _>>()?,

            speed: {
                info!("index: {}", self.index);
                let mut speed: Spline<f32> = self
                    .speed_key_points
                    .into_iter()
                    .map(|p| p.try_into())
                    .map(|p: Result<chart::KeyPoint<f32>, _>| {
                        let mut p = p?;
                        p.value = scale_y(p.value);
                        Ok(p)
                    })
                    .collect::<Result<_, _>>()?;
                let mut relevant_speed = speed.with_relevant::<f32>();
                let mut peekable = relevant_speed.points.iter_mut();
                while let Some(point) = peekable.next() {
                    let (this,next) = beat_to_time.pair(point.time + 0.01);
                    let this = this.unwrap();
                    let next = next.unwrap();
                    let mut k = (next.value - this.value) / (next.time- this.time);
                    if k.is_nan() {
                        k = 0.;
                    }
                    point.value *= k;
                }
                let speed = relevant_speed.with_relevant::<()>();
                speed
            },
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPoint {
    pub time: f32,

    pub value: f32,

    pub ease_type: u8,

    pub floor_position: f32,
}

impl TryInto<chart::KeyPoint<f32>> for KeyPoint {
    type Error = ConvertError;
    fn try_into(self) -> ConvertResult<chart::KeyPoint<f32>> {
        Ok(chart::KeyPoint {
            time: self.time,
            value: self.value,
            ease_type: self
                .ease_type
                .try_into()
                .or(Err(ConvertError::UnknownEaseKind {
                    raw_kind: self.ease_type,
                }))?,
            relevent: (),
        })
    }
}

// todo
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraMove {
    pub scale_key_points: Vec<KeyPoint>,

    pub x_position_key_points: Vec<KeyPoint>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RizlineChart {
    pub file_version: i32,

    pub songs_name: String,

    pub themes: [Theme; 2],

    pub challenge_times: Vec<ChallengeTime>,

    #[serde(rename = "bPM")]
    pub bpm: f32,

    pub bpm_shifts: Vec<KeyPoint>,

    pub offset: f32,

    pub lines: Vec<Line>,

    pub canvas_moves: Vec<CanvasMove>,

    pub camera_move: CameraMove,
}

impl TryInto<chart::Chart> for RizlineChart {
    type Error = ConvertError;

    fn try_into(self) -> ConvertResult<chart::Chart> {
        let [normal, challenge] = self.themes;
        let bpm = convert_bpm_to_timemap(self.bpm, self.bpm_shifts)?;
        let mut beat_cache = ChartCache::default();
        beat_cache.update_beat(&bpm);
        let beat_spline = beat_cache.beat.clone_reversed();
        info!("chart convert started");
        Ok(chart::Chart {
            themes: vec![normal.convert(false), challenge.convert(true)],
            // 如果challenge_times相互重叠(含 trans_time)则会产生奇怪的结果.
            theme_control: Some(chart::KeyPoint {
                time: 0.,
                value: 0,
                ..Default::default()
            })
            .into_iter()
            .chain(
                self.challenge_times
                    .into_iter()
                    .map(|c| {
                        [
                            chart::KeyPoint {
                                time: c.start - c.trans_time,
                                value: 0,
                                ..Default::default()
                            },
                            chart::KeyPoint {
                                time: c.start,
                                value: 1,
                                ..Default::default()
                            },
                            chart::KeyPoint {
                                time: c.end,
                                value: 1,
                                ..Default::default()
                            },
                            chart::KeyPoint {
                                time: c.end + c.trans_time,
                                value: 0,
                                ..Default::default()
                            },
                        ]
                        .into_iter()
                    })
                    .flatten(),
            )
            .collect(),

            lines: self
                .lines
                .into_iter()
                .enumerate()
                .map(|(index, line)| line.convert(index))
                .collect::<Result<Vec<_>, _>>()?,
            canvases: self
                .canvas_moves
                .into_iter()
                .map(|c| c.convert(&beat_spline))
                .collect::<ConvertResult<_>>()?,
            cam_move: self
                .camera_move
                .x_position_key_points
                .into_iter()
                .map(|mut k| {
                    k.value = scale_x(k.value);
                    k.try_into()
                })
                .collect::<ConvertResult<_>>()?,
            cam_scale: self
                .camera_move
                .scale_key_points
                .into_iter()
                .map(|k| k.try_into())
                .collect::<ConvertResult<_>>()?,
            bpm,
        })
    }
}

fn convert_bpm_to_timemap(bpm: f32, bpm_shifts: Vec<KeyPoint>) -> ConvertResult<Spline<f32>> {
    ensure!(!bpm_shifts.is_empty(), EmptyBPMSnafu);
    Ok(bpm_shifts
        .into_iter()
        .map(|s| chart::KeyPoint {
            time: s.time,
            value: bpm * s.value,
            ease_type: chart::EasingId::Start,
            relevent: (),
        })
        .collect())
}
