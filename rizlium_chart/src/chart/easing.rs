use crate::{Refc, Weak};
use std::fmt::Debug;
use std::ops::Add;

use log::{error, warn};
use simple_easing::*;

#[derive(Clone, Debug)]
pub struct KeyPointNext<T: Tween> {
    pub time: f32,
    pub value: T,
    pub ease_type: EasingId,
    pub relevant_ease: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SplineNext<T: Tween> {
    points: Vec<KeyPointNext<T>>,
}
impl<T: Tween> Default for SplineNext<T> {
    fn default() -> Self {
        Self { points: vec![] }
    }
}

impl<T: Tween> SplineNext<T> {
    pub fn points(&self) -> &Vec<KeyPointNext<T>> {
        &self.points
    }
    pub fn push(&mut self, keypoint: KeyPointNext<T>) {
        self.points.push(keypoint);
        self.sort_unstable();
    }

    pub fn remove(&mut self, index: usize) -> Option<KeyPointNext<T>> {
        if index < self.points.len() {

            Some(self.points.remove(index))
        }
        else {
            None
        }
    }

    pub fn sort_unstable(&mut self) {
        self.points.sort_unstable_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    pub fn push_many(&mut self, keypoints: impl IntoIterator<Item = KeyPointNext<T>>) {
        let iter = keypoints.into_iter();
        for keypoint in iter {
            self.points.push(keypoint);
        }
        self.sort_unstable();
    }
    pub fn iter(&self) -> impl Iterator<Item = &KeyPointNext<T>> {
        self.points.iter()
    }
}
impl<T: Tween> From<Vec<KeyPointNext<T>>> for SplineNext<T> {
    fn from(value: Vec<KeyPointNext<T>>) -> Self {
        let mut ret = Self { points: value };
        ret.sort_unstable();
        ret
    }
}

impl<T: Tween> FromIterator<KeyPointNext<T>> for SplineNext<T> {
    fn from_iter<I: IntoIterator<Item = KeyPointNext<T>>>(iter: I) -> Self {
        let mut ret: Self = Default::default();
        ret.push_many(iter);
        ret
    }
}
pub trait Tween: Clone {
    fn tween(x1: Self, x2: Self, t: f32) -> Self;
    fn ease(x1: Self, x2: Self, t: f32, easing: usize) -> Self {
        Self::tween(x1, x2, easef32(easing, t))
    }
}

impl Tween for f32 {
    fn tween(x1: Self, x2: Self, t: f32) -> Self {
        t * (x2 - x1) + x1
    }
}

pub type Easing = fn(f32) -> f32;
pub type EasingId = usize;

const EASING_MAP: [Easing; 16] = [
    linear,
    sine_in,
    sine_out,
    sine_in_out,
    quad_in,
    quad_out,
    quad_in_out,
    cubic_in,
    cubic_out,
    cubic_in_out,
    quart_in,
    quart_out,
    quart_in_out,
    |_t| 0.0,
    |_t| 1.0,
    |_t| {
        warn!("easing: easing 15(animCurve) is not supported");
        0.0
    },
];

fn easef32(ease_type: usize, x: f32) -> f32 {
    match EASING_MAP.get(ease_type) {
        Some(func) => func(x),
        None => {
            error!("Unknown ease type {}", ease_type);
            0.0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ease_basis() {
        assert_eq!(EASING_MAP[0](0.5), 0.5);
        assert_eq!(EASING_MAP[14](0.142857), 1.0);
    }
    #[test]
    fn test_lerp() {
        assert_eq!(f32::tween(0.2, 1.2, 0.9), 1.2);
    }
}
