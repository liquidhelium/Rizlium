use crate::chart::{self as original, EasingId, NoteKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<original::ColorRGBA> for ColorRGBA {
    fn from(src: original::ColorRGBA) -> Self {
        Self {
            r: src.r,
            g: src.g,
            b: src.b,
            a: src.a,
        }
    }
}

impl From<ColorRGBA> for original::ColorRGBA {
    fn from(src: ColorRGBA) -> Self {
        Self {
            r: src.r,
            g: src.g,
            b: src.b,
            a: src.a,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThemeColor {
    pub background: ColorRGBA,
    pub note: ColorRGBA,
    pub fx: ColorRGBA,
}

impl From<original::ThemeColor> for ThemeColor {
    fn from(src: original::ThemeColor) -> Self {
        Self {
            background: src.background.into(),
            note: src.note.into(),
            fx: src.fx.into(),
        }
    }
}

impl From<ThemeColor> for original::ThemeColor {
    fn from(src: ThemeColor) -> Self {
        Self {
            background: src.background.into(),
            note: src.note.into(),
            fx: src.fx.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThemeData {
    pub color: ThemeColor,
    pub is_challenge: bool,
}

impl From<original::ThemeData> for ThemeData {
    fn from(src: original::ThemeData) -> Self {
        Self {
            color: src.color.into(),
            is_challenge: src.is_challenge,
        }
    }
}

impl From<ThemeData> for original::ThemeData {
    fn from(src: ThemeData) -> Self {
        Self {
            color: src.color.into(),
            is_challenge: src.is_challenge,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyPoint<T, R = ()> {
    pub time: f32,
    pub value: T,
    pub ease_type: EasingId,
    #[serde(skip_serializing_if = "is_empty", default)]
    pub relevant: R,
}

fn is_empty<T>(_: &T) -> bool {
    std::mem::size_of::<T>() == 0
}

impl<To, Ro, Td, Rd> From<original::KeyPoint<To, Ro>> for KeyPoint<Td, Rd>
where
    Td: From<To>,
    Rd: From<Ro>,
    To: crate::chart::Tween,
{
    fn from(src: original::KeyPoint<To, Ro>) -> Self {
        Self {
            time: src.time,
            value: src.value.into(),
            ease_type: src.ease_type,
            relevant: src.relevant.into(),
        }
    }
}

impl<To, Ro, Td, Rd> From<KeyPoint<Td, Rd>> for original::KeyPoint<To, Ro>
where
    To: From<Td> + crate::chart::Tween,
    Ro: From<Rd>,
{
    fn from(src: KeyPoint<Td, Rd>) -> Self {
        Self {
            time: src.time,
            value: src.value.into(),
            ease_type: src.ease_type,
            relevant: src.relevant.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spline<T, R = ()> {
    #[serde(bound(
        deserialize = "Vec<KeyPoint<T, R>>: Deserialize<'de>",
        serialize = "R: Serialize, T: Serialize"
    ))]
    pub points: Vec<KeyPoint<T, R>>,
}

impl<To, Ro, Td, Rd> From<original::Spline<To, Ro>> for Spline<Td, Rd>
where
    Td: From<To>,
    Rd: From<Ro>,
    To: crate::chart::Tween,
{
    fn from(src: original::Spline<To, Ro>) -> Self {
        Self {
            points: src.points.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl<To, Ro, Td, Rd> From<Spline<Td, Rd>> for original::Spline<To, Ro>
where
    To: From<Td> + crate::chart::Tween,
    Ro: From<Rd>,
{
    fn from(src: Spline<Td, Rd>) -> Self {
        Self {
            points: src.points.into_iter().map(|p| p.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub time: f32,
    pub kind: NoteKind,
}

impl From<original::Note> for Note {
    fn from(src: original::Note) -> Self {
        Self {
            time: src.time,
            kind: src.kind,
        }
    }
}

impl From<Note> for original::Note {
    fn from(src: Note) -> Self {
        Self {
            time: src.time,
            kind: src.kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutNote {
    pub time: f32,
    pub x: f32,
    pub kind: NoteKind,
}

impl From<original::LayoutNote> for LayoutNote {
    fn from(src: original::LayoutNote) -> Self {
        Self {
            time: src.time,
            x: src.x,
            kind: src.kind,
        }
    }
}

impl From<LayoutNote> for original::LayoutNote {
    fn from(src: LayoutNote) -> Self {
        Self {
            time: src.time,
            x: src.x,
            kind: src.kind,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinePointData {
    pub canvas: usize,
    pub color: ColorRGBA,
}

impl From<original::LinePointData> for LinePointData {
    fn from(src: original::LinePointData) -> Self {
        Self {
            canvas: src.canvas,
            color: src.color.into(),
        }
    }
}

impl From<LinePointData> for original::LinePointData {
    fn from(src: LinePointData) -> Self {
        Self {
            canvas: src.canvas,
            color: src.color.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub points: Spline<f32, LinePointData>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub line_color: Spline<ColorRGBA>,
}

impl From<original::Line> for Line {
    fn from(src: original::Line) -> Self {
        Self {
            points: src.points.into(),
            notes: src.notes.into_iter().map(|n| n.into()).collect(),
            ring_color: src.ring_color.into(),
            line_color: src.line_color.into(),
        }
    }
}

impl From<Line> for original::Line {
    fn from(src: Line) -> Self {
        Self {
            name: String::new(),
            points: src.points.into(),
            notes: src.notes.into_iter().map(|n| n.into()).collect(),
            ring_color: src.ring_color.into(),
            line_color: src.line_color.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub x_pos: Spline<f32>,
    pub speed: Spline<f32>,
}

impl From<original::Canvas> for Canvas {
    fn from(src: original::Canvas) -> Self {
        Self {
            x_pos: src.x_pos.into(),
            speed: src.speed.into(),
        }
    }
}

impl From<Canvas> for original::Canvas {
    fn from(src: Canvas) -> Self {
        Self {
            name: String::new(),
            x_pos: src.x_pos.into(),
            speed: src.speed.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub themes: Vec<ThemeData>,
    pub theme_control: Spline<usize>,
    pub lines: Vec<Line>,
    pub canvases: Vec<Canvas>,
    pub bpm: Spline<f32>,
    pub cam_scale: Spline<f32>,
    pub cam_move: Spline<f32>,
    #[serde(default)]
    pub layout_notes: Vec<LayoutNote>,
}

impl From<original::Chart> for Chart {
    fn from(src: original::Chart) -> Self {
        Self {
            themes: src.themes.into_iter().map(|t| t.into()).collect(),
            theme_control: src.theme_control.into(),
            lines: src.lines.into_iter().map(|l| l.into()).collect(),
            canvases: src.canvases.into_iter().map(|c| c.into()).collect(),
            bpm: src.bpm.into(),
            cam_scale: src.cam_scale.into(),
            cam_move: src.cam_move.into(),
            layout_notes: src.layout_notes.into_iter().map(|n| n.into()).collect(),
        }
    }
}

impl From<Chart> for original::Chart {
    fn from(src: Chart) -> Self {
        Self {
            themes: src.themes.into_iter().map(|t| t.into()).collect(),
            theme_control: src.theme_control.into(),
            lines: src
                .lines
                .into_iter()
                .enumerate()
                .map(|(i, l)| {
                    let mut line: original::Line = l.into();
                    line.name = format!("Line {}", i + 1);
                    line
                })
                .collect(),
            canvases: src
                .canvases
                .into_iter()
                .enumerate()
                .map(|(i, c)| {
                    let mut canvas: original::Canvas = c.into();
                    canvas.name = format!("Canvas {}", i + 1);
                    canvas
                })
                .collect(),
            bpm: src.bpm.into(),
            cam_scale: src.cam_scale.into(),
            cam_move: src.cam_move.into(),
            layout_notes: src.layout_notes.into_iter().map(|n| n.into()).collect(),
        }
    }
}
