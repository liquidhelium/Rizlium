use crate::VIEW_RECT;

use crate::chart::{self, Spline};
use crate::parse::EmptyBPMSnafu;
use chart::LinePointData;
#[cfg(feature = "deserialize")]
use serde::Deserialize;
#[cfg(feature = "serialize")]
use serde::Serialize;
use snafu::{ensure, OptionExt};
use tracing::info;

use super::{ConvertError, ConvertResult, HoldNoEndSnafu};

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct ChallengeTime {
    pub check_point: f32,

    pub start: f32,

    pub end: f32,

    pub trans_time: f32,
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct Note {
    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "type")
    )]
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
                        .first()
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

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct ColorRGBA {
    pub r: u8,

    pub g: u8,

    pub b: u8,

    pub a: u8,
}

impl From<ColorRGBA> for chart::ColorRGBA {
    fn from(val: ColorRGBA) -> Self {
        Self {
            r: val.r as f32 / 255.0,
            g: val.g as f32 / 255.0,
            b: val.b as f32 / 255.0,
            a: val.a as f32 / 255.0,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct LinePoint {
    pub time: f32,

    pub x_position: f32,

    pub color: ColorRGBA,

    pub ease_type: u8,

    pub canvas_index: usize,

    pub floor_position: f32,
}

impl LinePoint {
    fn convert(self) -> ConvertResult<chart::KeyPoint<f32, chart::LinePointData>> {
        let color: chart::ColorRGBA = self.color.into();
        let point = chart::KeyPoint {
            time: self.time,
            value: self.x_position,
            ease_type: self
                .ease_type
                .try_into()
                .or(Err(ConvertError::UnknownEaseKind {
                    raw_kind: self.ease_type,
                }))?,
            relevant: LinePointData {
                canvas: self.canvas_index,
                color,
            },
        };
        Ok(point)
    }
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct ColorKeyPoint {
    pub start_color: ColorRGBA,

    pub end_color: ColorRGBA,

    pub time: f32,
}
impl From<ColorKeyPoint> for chart::KeyPoint<chart::ColorRGBA> {
    fn from(val: ColorKeyPoint) -> Self {
        Self {
            time: val.time,
            value: val.start_color.into(),
            ease_type: chart::EasingId::Linear,
            relevant: (),
        }
    }
}
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct Line {
    pub line_points: Vec<LinePoint>,

    pub notes: Vec<Note>,

    pub judge_ring_color: Vec<ColorKeyPoint>,

    pub line_color: Vec<ColorKeyPoint>,
}
impl Line {
    fn convert(self, line_index: usize) -> ConvertResult<chart::Line> {
        let line_color: Spline<_> = self.line_color.into_iter().map(Into::into).collect();
        let points = self
            .line_points
            .into_iter()
            .map(|p| p.convert())
            .collect::<ConvertResult<Vec<_>>>()?;
        let points: Spline<_, _> = points
            .into_iter()
            .map(|mut x| {
                x.value = scale_x(x.value);
                x
            })
            .collect();
        let notes: Vec<chart::Note> = self
            .notes
            .into_iter()
            .enumerate()
            .map(|(idx, n)| n.convert(line_index, idx))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(chart::Line {
            points,
            notes,
            ring_color: self.judge_ring_color.into_iter().map(Into::into).collect(),
            line_color,
        })
    }
}

fn scale_x(x: f32) -> f32 {
    x * (VIEW_RECT[1][0] - VIEW_RECT[0][0])
}
fn scale_y(y: f32) -> f32 {
    y * (VIEW_RECT[1][1] - VIEW_RECT[0][1]) * 1.
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct CanvasMove {
    pub index: i32,

    pub x_position_key_points: Vec<KeyPoint>,

    pub speed_key_points: Vec<KeyPoint>,
}

impl CanvasMove {
    fn convert(self) -> ConvertResult<chart::Canvas> {
        Ok(chart::Canvas {
            x_pos: self
                .x_position_key_points
                .into_iter()
                .map(TryInto::try_into)
                .map(|p: Result<chart::KeyPoint<f32>, _>| {
                    let mut p = p?;
                    p.value = scale_x(p.value);
                    Ok(p)
                })
                .collect::<Result<_, _>>()?,

            speed: self
                .speed_key_points
                .into_iter()
                .map(TryInto::try_into)
                .map(|p: Result<chart::KeyPoint<f32>, _>| {
                    let mut p = p?;
                    p.value = scale_y(p.value);
                    // linear here actually means constant start value
                    if p.ease_type == chart::EasingId::Linear {
                        p.ease_type = chart::EasingId::QuadOut;
                    }
                    Ok(p)
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
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
            relevant: (),
        })
    }
}

// todo
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct CameraMove {
    pub scale_key_points: Vec<KeyPoint>,

    pub x_position_key_points: Vec<KeyPoint>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct RizlineChart {
    pub file_version: i32,

    pub songs_name: String,

    pub themes: [Theme; 2],

    pub challenge_times: Vec<ChallengeTime>,

    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "bPM")
    )]
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
            .chain(self.challenge_times.into_iter().flat_map(|c| {
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
            }))
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
                .map(|c| c.convert())
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
                .map(TryInto::try_into)
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
            relevant: (),
        })
        .collect())
}
