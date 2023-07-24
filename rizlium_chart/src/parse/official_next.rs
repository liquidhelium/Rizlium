use crate::VIEW_RECT;

use crate::chart::{self, SplineNext};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Theme {
    #[serde(rename = "colorsList")]
    pub colors_list: Vec<ColorRGBA>,
}

#[derive(Serialize, Deserialize)]
pub struct ChallengeTime {
    #[serde(rename = "checkPoint")]
    pub check_point: f32,
    #[serde(rename = "start")]
    pub start: f32,
    #[serde(rename = "end")]
    pub end: f32,
    #[serde(rename = "transTime")]
    pub trans_time: f32,
}
#[derive(Serialize, Deserialize)]
pub struct Note {
    #[serde(rename = "type")]
    pub note_type: u8,
    #[serde(rename = "time")]
    pub time: f32,
    #[serde(rename = "floorPosition")]
    pub floor_position: f32,
    #[serde(rename = "otherInformations")]
    pub other_informations: Vec<f32>,
}
impl TryInto<chart::Note> for Note {
    type Error = super::ConvertError;
    fn try_into(self) -> Result<chart::Note, Self::Error> {
        Ok(chart::Note::new(
            self.time,
            match self.note_type {
                0 => chart::NoteKind::Tap,
                1 => chart::NoteKind::Drag,
                2 => chart::NoteKind::Hold {
                    end: *self
                        .other_informations
                        .get(0)
                        .ok_or(super::ConvertError("Hold without end time"))?,
                },
                _ => return Err(super::ConvertError("unknown type")),
            },
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct ColorRGBA {
    #[serde(rename = "r")]
    pub r: u8,
    #[serde(rename = "g")]
    pub g: u8,
    #[serde(rename = "b")]
    pub b: u8,
    #[serde(rename = "a")]
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
pub struct LinePoint {
    #[serde(rename = "time")]
    pub time: f32,
    #[serde(rename = "xPosition")]
    pub x_position: f32,
    #[serde(rename = "color")]
    pub color: ColorRGBA,
    #[serde(rename = "easeType")]
    pub ease_type: usize,
    #[serde(rename = "canvasIndex")]
    pub canvas_index: usize,
    #[serde(rename = "floorPosition")]
    pub floor_position: f32,
}

impl LinePoint {
    fn convert(
        self,
        line_index: usize,
    ) -> (
        chart::KeyPointNext<f32>,
        chart::KeyPointNext<chart::ColorRGBA>,
    ) {
        let point = chart::KeyPointNext {
            time: self.time,
            value: self.x_position,
            ease_type: self.ease_type,
            relevant_ease: Some(self.canvas_index),
        };
        let color = chart::KeyPointNext {
            time: self.time,
            value: self.color.into(),
            ease_type: self.ease_type,
            relevant_ease: Some(line_index),
        };
        (point, color)
    }
}
#[derive(Serialize, Deserialize)]
pub struct ColorKeyPoint {
    #[serde(rename = "startColor")]
    pub start_color: ColorRGBA,
    #[serde(rename = "endColor")]
    pub end_color: ColorRGBA,
    #[serde(rename = "time")]
    pub time: f32,
}
impl Into<chart::KeyPointNext<chart::ColorRGBA>> for ColorKeyPoint {
    fn into(self) -> chart::KeyPointNext<chart::ColorRGBA> {
        chart::KeyPointNext {
            time: self.time,
            value: self.start_color.into(),
            ease_type: 0,
            relevant_ease: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Line {
    #[serde(rename = "linePoints")]
    pub line_points: Vec<LinePoint>,
    #[serde(rename = "notes")]
    pub notes: Vec<Note>,
    #[serde(rename = "judgeRingColor")]
    pub judge_ring_color: Vec<ColorKeyPoint>,
    #[serde(rename = "lineColor")]
    pub line_color: Vec<ColorKeyPoint>,
}
impl Line {
    fn convert(self, line_index: usize) -> chart::LineNext {
        let line_color: SplineNext<_> = self.line_color.into_iter().map(|k| k.into()).collect();
        let (points, colors): (Vec<_>, Vec<_>) = self
            .line_points
            .into_iter()
            .map(|p| p.convert(line_index))
            .unzip();
        let points: SplineNext<_> = points
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
            .map(|n| n.try_into().unwrap())
            .collect();
        chart::LineNext {
            points,
            point_color: colors,
            notes,
            ring_color: self
                .judge_ring_color
                .into_iter()
                .map(|k| k.into())
                .collect(),
            line_color,
        }
    }
}

fn scale_x(x: f32) -> f32 {
    (x + 0.5) * (VIEW_RECT[1][0] - VIEW_RECT[0][0])
}
fn scale_y(y: f32) -> f32 {
    y * (VIEW_RECT[1][1] - VIEW_RECT[0][1]) * 0.5
}

#[derive(Serialize, Deserialize)]
pub struct CanvasMove {
    #[serde(rename = "index")]
    pub index: i32,
    #[serde(rename = "xPositionKeyPoints")]
    pub x_position_key_points: Vec<KeyPoint>,
    #[serde(rename = "speedKeyPoints")]
    pub speed_key_points: Vec<KeyPoint>,
}

impl Into<chart::Canvas> for CanvasMove {
    fn into(self) -> chart::Canvas {
        chart::Canvas {
            x_pos: self
                .x_position_key_points
                .into_iter()
                .map(|p| p.into())
                .map(|mut p: chart::KeyPointNext<f32>| {
                    p.value = scale_x(p.value);
                    p
                })
                .collect(),

            speed: self
                .speed_key_points
                .into_iter()
                .map(|p| p.into())
                .map(|mut p: chart::KeyPointNext<f32>| {
                    p.value = scale_y(p.value);
                    p
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct KeyPoint {
    #[serde(rename = "time")]
    pub time: f32,
    #[serde(rename = "value")]
    pub value: f32,
    #[serde(rename = "easeType")]
    pub ease_type: usize,
    #[serde(rename = "floorPosition")]
    pub floor_position: f32,
}

impl Into<chart::KeyPointNext<f32>> for KeyPoint {
    fn into(self) -> chart::KeyPointNext<f32> {
        chart::KeyPointNext {
            time: self.time,
            value: self.value,
            ease_type: self.ease_type,
            relevant_ease: None,
        }
    }
}

// todo
#[derive(Serialize, Deserialize)]
pub struct CameraMove {
    #[serde(rename = "scaleKeyPoints")]
    pub scale_key_points: Vec<KeyPoint>,
    #[serde(rename = "xPositionKeyPoints")]
    pub x_position_key_points: Vec<KeyPoint>,
}

#[derive(Serialize, Deserialize)]
pub struct RizlineChart {
    #[serde(rename = "fileVersion")]
    pub file_version: i32,
    #[serde(rename = "songsName")]
    pub songs_name: String,
    #[serde(rename = "themes")]
    pub themes: Vec<Theme>,
    #[serde(rename = "challengeTimes")]
    pub challenge_times: Vec<ChallengeTime>,
    #[serde(rename = "bPM")]
    pub bpm: f32,
    #[serde(rename = "bpmShifts")]
    pub bpm_shifts: Vec<KeyPoint>,
    #[serde(rename = "offset")]
    pub offset: f32,
    #[serde(rename = "lines")]
    pub lines: Vec<Line>,
    #[serde(rename = "canvasMoves")]
    pub canvas_moves: Vec<CanvasMove>,
    #[serde(rename = "cameraMove")]
    pub camera_move: CameraMove,
}

impl Into<chart::ChartNext> for RizlineChart {
    fn into(self) -> chart::ChartNext {
        let canvas = self.canvas_moves.into_iter().map(|c| c.into()).collect();
        let lines = self
            .lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| line.convert(index))
            .collect();
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
        chart::ChartNext {
            lines,
            canvases: canvas,
            cam_move,
            cam_scale,
            bpm: convert_bpm_to_timemap(self.bpm, self.bpm_shifts),
        }
    }
}

fn convert_bpm_to_timemap(bpm: f32, bpm_shifts: Vec<KeyPoint>) -> SplineNext<f32> {
    bpm_shifts
        .into_iter()
        .map(|s| chart::KeyPointNext {
            time: s.time,
            value: bpm * s.value,
            ease_type: 0,
            relevant_ease: None,
        })
        .collect()
}

