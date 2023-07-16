use crate::{Refc, Weak};
use std::fmt::Debug;
use std::ops::Add;

use log::{error, warn};
use nutype::nutype;
use simple_easing::*;

#[nutype(sanitize(with = |i| i.clamp(0.0,1.0)))]
#[derive(*,Deref)]
pub struct Clamped(f32);

#[nutype(validate(finite))]
#[derive(*)]
pub struct Finite(f32);

#[derive(Clone)]
pub struct KeyPoint<T: Tween> {
    pub time: f32,
    pub value: T,
    pub ease: EasingId,
    pub relevant_ease: Option<Weak<Spline<T>>>,
}

impl<T: Tween + Debug> Debug for KeyPoint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPoint")
            .field("time", &self.time)
            .field("value", &self.value)
            .field("ease", &self.ease)
            .field(
                "relevant_ease",
                &self
                    .relevant_ease
                    .as_ref()
                    .map(|e| e.upgrade().map(|s| s.id)),
            )
            .finish()
    }
}

impl<T: Tween> KeyPoint<T> {
    pub fn new(
        time: f32,
        value: T,
        ease: EasingId,
        relevant_ease: Option<Weak<Spline<T>>>,
    ) -> Self {
        Self {
            time,
            value,
            ease,
            relevant_ease,
        }
    }
}
impl<T: Tween + Add<Output = T>> KeyPoint<T> {
    fn may_offset(&self, offset: &Option<T>) -> T {
        offset
            .as_ref()
            .map(|o| o.clone() + self.value.clone())
            .unwrap_or(self.value.clone())
    }
    fn get_offset(&self, time: f32, relevant_ease_time: f32) -> Option<T> {
        self.get_relevant()
            .map(|l| l.value_at_related(time, relevant_ease_time))
    }
    pub fn get_relevant(&self) -> Option<Refc<Spline<T>>> {
        self.relevant_ease.as_ref().map(|l| l.upgrade()).flatten()
    }
    pub fn related_value(&self, game_time: f32) -> T {
        // 以后有多重的再改
        self.may_offset(&self.get_offset(game_time, 0.0))
    }
}

#[derive(Debug, Clone)]
pub struct Spline<T: Tween> {
    id: u32,
    pub points: Vec<KeyPoint<T>>,
}
impl<T: Tween> PartialEq for Spline<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T: Tween> Spline<T> {
    pub fn new(id: u32, points: Vec<KeyPoint<T>>) -> Self {
        Self { id, points }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn try_find_by_time(&self, time: f32) -> Option<usize> {
        let (Ok(find) | Err(find)) = self
            .points
            .binary_search_by_key(&(Finite::new(time).unwrap()), |k| {
                Finite::new(k.time).unwrap()
            });
        find.checked_add_signed(-1)
    }
    pub fn find_by_time(&self, time: f32) -> usize {
        self.try_find_by_time(time).unwrap_or(0)
    }

    pub fn value_at(&self, time: f32) -> T {
        let index = self.find_by_time(time);
        let former = self.points.get(index).unwrap_or(
            self.points
                .last()
                .expect(&format!("Spline {} appear to be empty", self.id)),
        );
        match self.points.get(index + 1) {
            Some(latter) => {
                let t = (time - former.time) / (latter.time - former.time);
                T::ease(
                    former.value.clone(),
                    latter.value.clone(),
                    Clamped::new(t),
                    former.ease,
                )
            }
            None => former.value.clone(),
        }
    }
}

impl<T: Tween + std::ops::Add<Output = T>> Spline<T> {
    pub fn try_value_at_related(&self, time: f32, relevant_ease_time: f32) -> Option<T> {
        let index = self.find_by_time(time);
        let former = self
            .points
            .get(index)
            .expect("unexpected wrong index out of range");
        let offset1 = former.get_offset(relevant_ease_time, 0.);
        match self.points.get(index + 1) {
            Some(latter) => {
                let mut t = (time - former.time) / (latter.time - former.time);
                if t.is_nan() {
                    t = 0.;
                }
                if former.get_relevant() == latter.get_relevant() {
                    Some(T::ease(
                        former.may_offset(&offset1),
                        latter.may_offset(&offset1),
                        Clamped::new(t),
                        former.ease,
                    ))
                } else {
                    let offset2 = latter.get_offset(time, relevant_ease_time);
                    Some(T::ease(
                        former.may_offset(&offset1),
                        latter.may_offset(&offset2),
                        Clamped::new(t),
                        former.ease,
                    ))
                }
            }
            None => None,
        }
    }
    pub fn value_at_related(&self, time: f32, relevant_ease_time: f32) -> T {
        self.try_value_at_related(time, relevant_ease_time)
            .unwrap_or_else(|| {
                if time < self.points.first().map_or(0.0, |p| p.time) {
                    self.points[0].value.clone()
                } else {
                    self.points.last().expect("empty spline").value.clone()
                }
            })
    }
}

pub trait Tween: Clone {
    fn tween(x1: Self, x2: Self, t: Clamped) -> Self;
    fn ease(x1: Self, x2: Self, t: Clamped, easing: usize) -> Self {
        Self::tween(x1, x2, Clamped::new(easef32(easing, *t)))
    }
}

impl Tween for f32 {
    fn tween(x1: Self, x2: Self, t: Clamped) -> Self {
        *t * (x2 - x1) + x1
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
        assert_eq!(f32::tween(0.2, 1.2, Clamped::new(0.9)), 1.2);
    }
}
