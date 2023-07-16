use crate::{Refc, VIEW_RECT};

use crate::chart::{self, Spline};
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
    pub note_type: i32,
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
    pub r: i32,
    #[serde(rename = "g")]
    pub g: i32,
    #[serde(rename = "b")]
    pub b: i32,
    #[serde(rename = "a")]
    pub a: i32,
}

impl Into<chart::ColorRGBA> for ColorRGBA {
    fn into(self) -> chart::ColorRGBA {
        chart::ColorRGBA {
            r: self.r as f32,
            g: self.g as f32,
            b: self.b as f32,
            a: self.a as f32,
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
        line_color: Refc<chart::Spline<chart::ColorRGBA>>,
        canvas: &Vec<Refc<chart::Spline<f32>>>,
    ) -> (chart::KeyPoint<f32>, chart::KeyPoint<chart::ColorRGBA>) {
        let point = chart::KeyPoint::new(
            self.time,
            self.x_position,
            self.ease_type,
            canvas.get(self.canvas_index).map(|rc| Refc::downgrade(rc)),
        );
        let color = chart::KeyPoint::new(
            self.time,
            self.color.into(),
            0,
            Some(Refc::downgrade(&line_color)),
        );
        (point, color)
    }
}
#[derive(Serialize, Deserialize)]
pub struct JudgeRingColor {
    #[serde(rename = "startColor")]
    pub start_color: ColorRGBA,
    #[serde(rename = "endColor")]
    pub end_color: ColorRGBA,
    #[serde(rename = "time")]
    pub time: f32,
}
impl Into<chart::KeyPoint<chart::ColorRGBA>> for JudgeRingColor {
    fn into(self) -> chart::KeyPoint<chart::ColorRGBA> {
        chart::KeyPoint::new(self.time, self.start_color.into(), 0, None)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Line {
    #[serde(rename = "linePoints")]
    pub line_points: Vec<LinePoint>,
    #[serde(rename = "notes")]
    pub notes: Vec<Note>,
    #[serde(rename = "judgeRingColor")]
    pub judge_ring_color: Vec<JudgeRingColor>,
    #[serde(rename = "lineColor")]
    pub line_color: Vec<ColorRGBA>,
}
impl Line {
    fn convert(
        self,
        canvas: &Vec<Refc<chart::Spline<f32>>>,
        index: usize,
        vmove: Refc<Spline<f32>>,
    ) -> chart::Line {
        let line_color: Vec<chart::ColorRGBA> =
            self.line_color.into_iter().map(|k| k.into()).collect();
        let line_color = Refc::new(Spline::new(
            index.try_into().unwrap(),
            line_color
                .into_iter()
                .map(|c| chart::KeyPoint::new(0.0, c, 0, None))
                .collect(),
        ));
        let (points, colors): (Vec<_>, Vec<_>) = self
            .line_points
            .into_iter()
            .map(|p| p.convert(Refc::clone(&line_color), canvas))
            .unzip();
        let points = points
            .into_iter()
            .map(|mut x| {
                x.value = scale_x(x.value);
                x
            })
            .collect();
        let points = chart::Spline::new(index as u32, points);
        let color = chart::Spline::new(u32::MAX - index as u32, colors);
        // todo: color mix
        let notes: Vec<chart::Note> = self
            .notes
            .into_iter()
            .map(|n| n.try_into().unwrap())
            .collect();
        chart::Line::new(
            points,
            color,
            notes,
            Spline::new(
                index.try_into().unwrap(),
                self.judge_ring_color
                    .into_iter()
                    .map(|c| c.into())
                    .collect(),
            ),
            vmove,
            line_color,
        )
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

impl Into<(chart::Spline<f32>, chart::Spline<f32>)> for CanvasMove {
    fn into(self) -> (chart::Spline<f32>, chart::Spline<f32>) {
        (
            Spline::new(
                self.index as u32,
                self.x_position_key_points
                    .into_iter()
                    .map(|p| p.into())
                    .map(|mut p: chart::KeyPoint<f32>| {
                        p.value = scale_x(p.value);
                        p
                    })
                    .collect(),
            ),
            Spline::new(
                self.index as u32,
                self.speed_key_points
                    .into_iter()
                    .map(|p| chart::KeyPoint::new(p.time, p.floor_position -0.5, 0, None))
                    .map(|mut p| {
                        p.value = scale_y(p.value);
                        p
                    })
                    .collect(),
            ),
        )
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

impl Into<chart::KeyPoint<f32>> for KeyPoint {
    fn into(self) -> chart::KeyPoint<f32> {
        chart::KeyPoint::new(self.time, self.value, self.ease_type, None)
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

impl Into<chart::RizChart> for RizlineChart {
    fn into(self) -> chart::RizChart {
        let (canvas, vmove): (Vec<_>, Vec<_>) =
            self.canvas_moves.into_iter().map(|c| c.into()).unzip();
        let canvas = canvas.into_iter().map(|l| Refc::new(l)).collect();
        let vmove: Vec<_> = vmove.into_iter().map(|v| Refc::new(v)).collect();
        let lines = self
            .lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                let canvas_index = line.line_points[0].canvas_index;
                line.convert(&canvas, index, Refc::clone(&vmove[canvas_index]))
            })
            .collect();
        chart::RizChart::new(lines, canvas, convert_bpm(self.bpm, self.bpm_shifts))
    }
}

fn convert_bpm(bpm: f32, bpm_shifts: Vec<KeyPoint>) -> Spline<f32> {
    let bpms = bpm_shifts
        .into_iter()
        .map(|s| chart::KeyPoint::new(s.time, s.value * bpm, 0, None))
        .collect();
    Spline::new(114514, bpms)
}
