use log::{error, warn};
use simple_easing::*;



#[derive(Clone, Debug, Default)]
pub struct KeyPoint<T: Tween> {
    pub time: f32,
    pub value: T,
    pub ease_type: EasingId,
    pub relevant_ease: Option<usize>,
}

impl<T: Tween> KeyPoint<T> {
    pub fn ease_to(&self,next: &KeyPoint<T>, t: f32) -> T {
        T::ease(self.value.clone(), next.value.clone(), t, self.ease_type)
    }
}


/// Structure that defines how a value varies.
#[derive(Debug, Clone)]
pub struct Spline<T: Tween> {
    points: Vec<KeyPoint<T>>,
}

impl<T:Tween> Spline<T> {
    /// Value of `self` at `time`, `None` when `self` is empty, keep the value when overflow or underflow.
    pub fn value_padding(&self, time: f32) -> Option<T> {
        match self.pair(time) {
            (Some(curr), Some(next)) => {
                let mut t =  (time - curr.time) / (next.time - curr.time);
                if t.is_nan() {
                    t = 0.0
                }
                Some(curr.ease_to(&next, t))
            },
            (Some(last), None) => Some(last.value.clone()),
            (None, Some(first)) => Some(first.value.clone()),
            (None, None) => None,
        }
    }
    /// Value of `self` at `time`, `None` when `time` out of bounds or `self` is empty.
    pub fn value_ignoring(&self, time: f32) -> Option<T> {
        match self.pair(time) {
            (Some(curr), Some(next)) => {
                let mut t =  (time - curr.time) / (next.time - curr.time);
                if t.is_nan() {
                    t = 0.0
                }
                Some(curr.ease_to(&next, t))
            },
            _ => None
        }
    }
}

/// Find
impl<T: Tween> Spline<T> {
    /// Return a pair of [`KeyPoint`] at this `time`.
    /// 
    /// When this [`Spline`] is not empty, returns:
    ///  - `(Some(_), Some(_))` when time is between `start_time..end_time`
    ///  - `(Some(_), None)` when `time >= end_time`
    ///  - `(None, Some(_))` when `time < start_time`
    /// 
    /// Returns `(None, None)` when empty.
    /// 
    /// ## Examples
    /// Empty:
    /// ```rust
    /// use rizlium_chart::chart::Spline;
    /// let spline = Spline::<f32>::default();
    /// assert!(matches!(spline.pair(0.0), (None, None)));
    /// assert!(matches!(spline.pair(114514.0), (None, None)));
    /// ```
    /// Single:
    /// ```rust
    /// use rizlium_chart::chart::KeyPoint;
    /// use rizlium_chart::chart::Spline;
    /// let spline:Spline<f32> = vec![KeyPoint::default()].into();
    /// assert!(matches!(spline.pair(-1.0), (None, Some(_))));
    /// assert!(matches!(spline.pair(1.0), (Some(_), None)));
    /// ```
    /// Two:
    /// ```rust
    /// use rizlium_chart::chart::KeyPoint;
    /// use rizlium_chart::chart::Spline;
    /// let spline:Spline<f32> = vec![KeyPoint {
    ///     time: 0.0,
    ///     ..Default::default()
    /// },KeyPoint {
    ///     time: 2.0,
    ///     ..Default::default()
    /// }].into();
    /// assert!(matches!(spline.pair(-1.0), (None, Some(_))));
    /// assert!(matches!(spline.pair(1.0), (Some(_), Some(_))));
    /// assert!(matches!(spline.pair(2.2), (Some(_), None)));
    /// ```
    pub fn pair(&self, time: f32) -> (Option<&KeyPoint<T>>, Option<&KeyPoint<T>>) {
        match self.keypoint_at(time) {
            Ok(index) => (self.points.get(index), self.points.get(index+1)),
            Err(index) => if index == 0 {
                (None, self.points.first())
            } 
            else {
                (self.points.last(), None)
            }
        }
    }
    /// Return which [`KeyPoint`] the `time` is in.
    /// 
    /// `Ok(val)` is between `0..=self.points.len() - 2`;  
    /// 
    /// `Err(val)` is either `0` or `self.points.len()`.  
    /// 
    /// If this [`Spline`] is empty then `Err(0)` is returned.
    pub fn keypoint_at(&self, time: f32) -> Result<usize, usize> {
        let (Ok(val) | Err(val)) = self.points.binary_search_by(|p| {
            p.time.partial_cmp(&time).unwrap_or(std::cmp::Ordering::Equal)
        });
        if val == 0 || val == self.points.len(){
            Err(val)
        }
        else {
            // SAFETY: val != 0
            Ok(val.wrapping_add_signed(-1))
        }
    }
    /// Start time of this [`Spline`], return `None` if this [`Spline`] is empty.
    pub fn start_time(&self) -> Option<f32> {
        self.points.first().map(|p| p.time)
    }
    /// End time of this [`Spline`], return `None` if this [`Spline`] is empty.
    pub fn end_time(&self) -> Option<f32> {
        self.points.last().map(|p| p.time)
    }
}

impl<T: Tween> Default for Spline<T> {
    fn default() -> Self {
        Self { points: vec![] }
    }
}

/// # Mutations
impl<T: Tween> Spline<T> {
    pub fn points(&self) -> &Vec<KeyPoint<T>> {
        &self.points
    }
    pub fn push(&mut self, keypoint: KeyPoint<T>) {
        self.points.push(keypoint);
        self.sort_unstable();
    }

    pub fn remove(&mut self, index: usize) -> Option<KeyPoint<T>> {
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
    pub fn push_many(&mut self, keypoints: impl IntoIterator<Item = KeyPoint<T>>) {
        let iter = keypoints.into_iter();
        for keypoint in iter {
            self.points.push(keypoint);
        }
        self.sort_unstable();
    }
    pub fn iter(&self) -> impl Iterator<Item = &KeyPoint<T>> {
        self.points.iter()
    }
}
impl<T: Tween> From<Vec<KeyPoint<T>>> for Spline<T> {
    fn from(value: Vec<KeyPoint<T>>) -> Self {
        let mut ret = Self { points: value };
        ret.sort_unstable();
        ret
    }
}

impl<T: Tween> FromIterator<KeyPoint<T>> for Spline<T> {
    fn from_iter<I: IntoIterator<Item = KeyPoint<T>>>(iter: I) -> Self {
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
