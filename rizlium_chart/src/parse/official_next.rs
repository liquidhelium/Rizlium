use crate::VIEW_RECT;

use crate::chart::{self, Spline};
use crate::parse::EmptyBPMSnafu;
use serde_derive::{Deserialize, Serialize};
use snafu::{OptionExt, ensure};

use super::{ConvertError, HoldNoEndSnafu, ConvertResult, UnknownNoteKindSnafu};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    
    pub colors_list: Vec<ColorRGBA>,
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
    #[serde(rename="type")]
    pub note_type: u8,
    
    pub time: f32,
    
    pub floor_position: f32,
    
    pub other_informations: Vec<f32>,
}
impl Note {
    fn convert(self, line_idx:usize, note_idx:usize) -> ConvertResult<chart::Note> {
        Ok(chart::Note::new(
            self.time,
            match self.note_type {
                0 => chart::NoteKind::Tap,
                1 => chart::NoteKind::Drag,
                2 => chart::NoteKind::Hold {
                    end: *self
                        .other_informations
                        .get(0)
                        .with_context(|| HoldNoEndSnafu { line_idx, note_idx})?,
                },
                otherwise => return Err(ConvertError::UnknownNoteKind   { raw_kind: otherwise as usize }),
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
    
    pub ease_type: usize,
    
    pub canvas_index: usize,
    
    pub floor_position: f32,
}

impl LinePoint {
    fn convert(
        self,
        line_index: usize,
    ) -> (
        chart::KeyPoint<f32>,
        chart::KeyPoint<chart::ColorRGBA>,
    ) {
        let point = chart::KeyPoint {
            time: self.time,
            value: self.x_position,
            ease_type: self.ease_type,
            relevant_ease: Some(self.canvas_index),
        };
        let color = chart::KeyPoint {
            time: self.time,
            value: self.color.into(),
            ease_type: self.ease_type,
            relevant_ease: Some(line_index),
        };
        (point, color)
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
            ease_type: 0,
            relevant_ease: None,
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
        let (points, colors): (Vec<_>, Vec<_>) = self
            .line_points
            .into_iter()
            .map(|p| p.convert(line_index))
            .unzip();
        let points: Spline<_> = points
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
            .map(|(idx,n)| n.convert(line_index, idx))
            .collect::<Result<Vec<_>, _>>()?;
       Ok( chart::Line {
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
    y * (VIEW_RECT[1][1] - VIEW_RECT[0][1]) * 0.5
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasMove {
    
    pub index: i32,
    
    pub x_position_key_points: Vec<KeyPoint>,
    
    pub speed_key_points: Vec<KeyPoint>,
}

impl Into<chart::Canvas> for CanvasMove {
    fn into(self) -> chart::Canvas {
        chart::Canvas {
            x_pos: self
                .x_position_key_points
                .into_iter()
                .map(|p| p.into())
                .map(|mut p: chart::KeyPoint<f32>| {
                    p.value = scale_x(p.value);
                    p
                })
                .collect(),

            speed: self
                .speed_key_points
                .into_iter()
                .map(|p| p.into())
                .map(|mut p: chart::KeyPoint<f32>| {
                    p.value = scale_y(p.value);
                    p
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPoint {
    
    pub time: f32,
    
    pub value: f32,
    
    pub ease_type: usize,
    
    pub floor_position: f32,
}

impl Into<chart::KeyPoint<f32>> for KeyPoint {
    fn into(self) -> chart::KeyPoint<f32> {
        chart::KeyPoint {
            time: self.time,
            value: self.value,
            ease_type: self.ease_type,
            relevant_ease: None,
        }
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
    
    pub themes: Vec<Theme>,
    
    pub challenge_times: Vec<ChallengeTime>,

    #[serde(rename="bPM")]
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
        let canvas = self.canvas_moves.into_iter().map(|c| c.into()).collect();
        let lines = self
            .lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| line.convert(index))
            .collect::<Result<Vec<_>,_>>()?;
        let cam_scale = self
            .camera_move
            .scale_key_points
            .into_iter()
            .map(|k| k.into())
            .collect();
        let cam_move = self
            .camera_move
            .x_position_key_points
            .into_iter()
            .map(|mut k| {
                k.value = scale_x(k.value);
                k.into()
            })
            .collect();
        Ok(chart::Chart {
                    lines,
                    canvases: canvas,
                    cam_move,
                    cam_scale,
                    bpm: convert_bpm_to_timemap(self.bpm, self.bpm_shifts)?,
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
                ease_type: 0,
                relevant_ease: None,
            })
            .collect())
}

