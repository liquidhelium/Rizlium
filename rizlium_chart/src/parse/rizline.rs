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

#[derive(Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct RizlineChartMeta {
    pub title: String,
    pub composer: String,
    pub difficulty: f32,
    pub level: f32,
    pub max_hit: usize,
    pub max_score: usize,
    pub preview_time: f32
}

#[derive(Clone)]
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
        let [bg, note, fx] = self.colors_list;
        chart::ThemeData {
            color: chart::ThemeColor {
                background: bg.into(),
                note: note.into(),
                fx: fx.into(),
            },
            is_challenge,
        }
    }
}

#[derive(Clone)]
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
#[derive(Clone)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
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
        let line_color: Spline<_> = self
            .line_color
            .windows(2)
            .flat_map(|arr| {
                let a = &arr[0];
                let b = &arr[1];
                vec![
                    chart::KeyPoint {
                        time: a.time,
                        value: a.start_color.into(),
                        ease_type: chart::EasingId::Linear,
                        relevant: (),
                    },
                    chart::KeyPoint {
                        time: b.time,
                        value: a.end_color.into(),
                        ease_type: chart::EasingId::Linear,
                        relevant: (),
                    },
                ]
            })
            .chain(self.line_color.last().map(|c| chart::KeyPoint {
                time: c.time,
                value: c.end_color.into(),
                ease_type: chart::EasingId::Linear,
                relevant: (),
            }))
            .collect();
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
            name: format!("Line {}", line_index + 1),
            points,
            notes,
            ring_color: self
                .judge_ring_color
                .windows(2)
                .flat_map(|arr| {
                    let a = &arr[0];
                    let b = &arr[1];
                    vec![
                        chart::KeyPoint {
                            time: a.time,
                            value: a.start_color.into(),
                            ease_type: chart::EasingId::Linear,
                            relevant: (),
                        },
                        chart::KeyPoint {
                            time: b.time,
                            value: a.end_color.into(),
                            ease_type: chart::EasingId::Linear,
                            relevant: (),
                        },
                    ]
                })
                .chain(self.judge_ring_color.last().map(|c| chart::KeyPoint {
                    time: c.time,
                    value: c.end_color.into(),
                    ease_type: chart::EasingId::Linear,
                    relevant: (),
                }))
                .collect(),
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

#[derive(Clone)]
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
        let name = format!("Canvas {}", self.index + 1);
        Ok(chart::Canvas {
            name,
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

#[derive(Clone)]
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
#[derive(Clone)]
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

#[derive(Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(rename_all = "camelCase")
)]
pub struct RizlineChart {
    pub file_version: i32,
    #[serde(default)]
    pub songs_name: String,

    pub themes: Vec<Theme>,

    pub challenge_times: Vec<ChallengeTime>,

    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "bPM")
    )]
    pub bpm: f32,

    pub bpm_shifts: Vec<KeyPoint>,
    #[serde(alias = "chartDelayMs")]
    pub offset: f32,

    pub lines: Vec<Line>,

    pub canvas_moves: Vec<CanvasMove>,

    pub camera_move: CameraMove,
}

impl TryInto<chart::Chart> for RizlineChart {
    type Error = ConvertError;

    fn try_into(self) -> ConvertResult<chart::Chart> {
        let bpm = convert_bpm_to_timemap(self.bpm, self.bpm_shifts)?;
        info!("chart convert started");
        Ok(chart::Chart {
            themes: self.themes.into_iter().enumerate().map(|(i, t)| {
                if i == 0 {
                    t.convert(false)
                } else {
                    t.convert(true)
                }
            }).collect(),
            // 如果challenge_times相互重叠(含 trans_time)则会产生奇怪的结果.
            theme_control: Some(chart::KeyPoint {
                time: 0.,
                value: 0,
                ..Default::default()
            })
            .into_iter()
            .chain(self.challenge_times.into_iter().enumerate().flat_map(|(i, c)| {
                [
                    chart::KeyPoint {
                        time: c.start - c.trans_time,
                        value: 0,
                        ..Default::default()
                    },
                    chart::KeyPoint {
                        time: c.start,
                        value: i+1,
                        ..Default::default()
                    },
                    chart::KeyPoint {
                        time: c.end,
                        value: i+1,
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
            layout_notes: vec![],
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

fn unscale_x(x: f32) -> f32 {
    x / (VIEW_RECT[1][0] - VIEW_RECT[0][0])
}
fn unscale_y(y: f32) -> f32 {
    y / (VIEW_RECT[1][1] - VIEW_RECT[0][1])
}

impl From<chart::ColorRGBA> for ColorRGBA {
    fn from(val: chart::ColorRGBA) -> Self {
        Self {
            r: (val.r * 255.0) as u8,
            g: (val.g * 255.0) as u8,
            b: (val.b * 255.0) as u8,
            a: (val.a * 255.0) as u8,
        }
    }
}

impl From<chart::ThemeData> for Theme {
    fn from(val: chart::ThemeData) -> Self {
        Self {
            colors_list: [
                val.color.background.into(),
                val.color.note.into(),
                val.color.fx.into(),
            ],
        }
    }
}

impl From<chart::KeyPoint<f32, chart::LinePointData>> for LinePoint {
    fn from(kp: chart::KeyPoint<f32, chart::LinePointData>) -> Self {
        Self {
            time: kp.time,
            x_position: unscale_x(kp.value),
            color: kp.relevant.color.into(),
            ease_type: kp.ease_type.into(),
            canvas_index: kp.relevant.canvas,
            floor_position: 0.0,
        }
    }
}

impl From<chart::Note> for Note {
    fn from(note: chart::Note) -> Self {
        let (note_type, other_informations) = match note.kind {
            chart::NoteKind::Tap => (0, vec![]),
            chart::NoteKind::Drag => (1, vec![]),
            chart::NoteKind::Hold { end } => (2, vec![end]),
        };

        Self {
            note_type,
            time: note.time,
            floor_position: 0.0,
            other_informations,
        }
    }
}

impl From<chart::Line> for Line {
    fn from(line: chart::Line) -> Self {
        let line_points = line.points.points.into_iter().map(Into::into).collect();
        let notes = line.notes.into_iter().map(Into::into).collect();

        let convert_color_spline = |spline: &Spline<chart::ColorRGBA>| -> Vec<ColorKeyPoint> {
            let points = &spline.points;
            if points.is_empty() {
                return vec![];
            }

            let mut result = Vec::new();
            for i in 0..points.len() {
                let p = &points[i];
                let next_val = if i + 1 < points.len() {
                    points[i+1].value
                } else {
                    p.value
                };

                result.push(ColorKeyPoint {
                    time: p.time,
                    start_color: p.value.into(),
                    end_color: next_val.into(),
                });
            }
            result
        };

        Self {
            line_points,
            notes,
            judge_ring_color: convert_color_spline(&line.ring_color),
            line_color: convert_color_spline(&line.line_color),
        }
    }
}

impl CanvasMove {
    fn from_chart_canvas(index: i32, canvas: &chart::Canvas) -> Self {
        Self {
            index,
            x_position_key_points: canvas.x_pos.points.iter().map(|p| KeyPoint {
                time: p.time,
                value: unscale_x(p.value),
                ease_type: p.ease_type.into(),
                floor_position: 0.0,
            }).collect(),
            speed_key_points: canvas.speed.points.iter().map(|p| KeyPoint {
                time: p.time,
                value: unscale_y(p.value),
                ease_type: p.ease_type.into(),
                floor_position: 0.0,
            }).collect(),
        }
    }
}

impl TryFrom<chart::Chart> for RizlineChart {
    type Error = ConvertError;

    fn try_from(chart: chart::Chart) -> Result<Self, Self::Error> {

        let mut challenge_times = Vec::new();
        let mut current_theme_idx = 0;
        let mut start_time = None;

        for point in chart.theme_control.points() {
             if point.value != current_theme_idx {
                 if point.value != 0 { // Assuming != 0 is challenge
                     if start_time.is_none() {
                        start_time = Some(point.time);
                     }
                 } else if start_time.is_some() {
                     challenge_times.push(ChallengeTime {
                         check_point: 0.0,
                         start: start_time.unwrap(),
                         end: point.time,
                         trans_time: 1.0,
                     });
                     start_time = None;
                 }
                 current_theme_idx = point.value;
             }
        }

        if let Some(start) = start_time {
             let max_time = chart.lines.iter()
                .flat_map(|l| l.points.points().iter().map(|p| p.time))
                .fold(0.0f32, f32::max);
             challenge_times.push(ChallengeTime {
                 check_point: 0.0,
                 start,
                 end: max_time.max(start + 1.0),
                 trans_time: 1.0,
             });
        }

        if challenge_times.is_empty() {
             challenge_times.push(ChallengeTime {
                 check_point: 0.0,
                 start: 0.0,
                 end: 1.0,
                 trans_time: 1.0,
             });
        }

        let base_bpm = chart.bpm.points().first().map(|p| p.value).unwrap_or(120.0);
        let bpm_shifts = chart.bpm.points().iter().map(|p| {
            KeyPoint {
                time: p.time,
                value: p.value / base_bpm,
                ease_type: 0,
                floor_position: 0.0,
            }
        }).collect();

        let lines = chart.lines.into_iter().map(Into::into).collect();

        let canvas_moves = chart.canvases.iter().enumerate().map(|(i, c)| {
             CanvasMove::from_chart_canvas(i as i32, c)
        }).collect();

        let camera_move = CameraMove {
            scale_key_points: chart.cam_scale.points.iter().map(|p| KeyPoint {
                time: p.time,
                value: p.value,
                ease_type: p.ease_type.into(),
                floor_position: 0.0,
            }).collect(),
            x_position_key_points: chart.cam_move.points.iter().map(|p| {
                KeyPoint {
                    time: p.time,
                    value: unscale_x(p.value),
                    ease_type: p.ease_type.into(),
                    floor_position: 0.0,
                }
            }).collect(),
        };

        Ok(RizlineChart {
            file_version: 1,
            songs_name: "Untitled".to_string(),
            themes: chart.themes.into_iter().map(Into::into).collect(),
            challenge_times,
            bpm: base_bpm,
            bpm_shifts,
            offset: 0.0,
            lines,
            canvas_moves,
            camera_move,
        })
    }
}
