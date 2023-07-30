use log::{error, warn};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use simple_easing::*;
#[macro_export]
macro_rules! tween {
    (($($var:ident),*),$x1:ident,$x2:ident, $t:ident) => {
        Self {
            $($var: Tween::lerp($x1.$var,$x2.$var,$t),)*
        }
    };
}
pub fn invlerp(y1: f32, y2: f32, y0: f32) -> f32 {
    let t = (y0 - y1) / (y2 - y1);
    if t.is_nan() {
        0.
    }
    else {
        t
    }
}

#[derive(Clone, Debug, Default)]
pub struct KeyPoint<T: Tween, R = ()> {
    pub time: f32,
    pub value: T,
    pub ease_type: EasingId,
    pub relevent: R,
}

impl<T: Tween, R> KeyPoint<T, R> {
    pub fn ease_to(&self, next: &KeyPoint<T, R>, t: f32) -> T {
        T::ease(self.value.clone(), next.value.clone(), t, self.ease_type)
    }
}

/// 用于平缓地更改一个值.
#[derive(Debug, Clone)]
pub struct Spline<T: Tween, R= ()> {
    points: Vec<KeyPoint<T, R>>,
}

impl<T: Tween> Spline<T> {
    /// 该 [`Spline`] 在 `time` 时间的值.
    ///
    /// 如果该 [`Spline`] 是空的, 返回 `None`.
    ///
    /// 如果时间不在这条线的范围内则保持值不变, 返回最后一个/第一个值.
    pub fn value_padding(&self, time: f32) -> Option<T> {
        match self.pair(time) {
            (Some(curr), Some(next)) => {
                let t = invlerp(curr.time, next.time, time);
                Some(curr.ease_to(&next, t))
            }
            (Some(last), None) => Some(last.value.clone()),
            (None, Some(first)) => Some(first.value.clone()),
            (None, None) => None,
        }
    }
    /// 该 [`Spline`] 在 `time` 时间的值, `time` 出界或此线为空时返回 `None`.
    pub fn value(&self, time: f32) -> Option<T> {
        match self.pair(time) {
            (Some(curr), Some(next)) => {
                let t = invlerp(curr.time, next.time, time);
                Some(curr.ease_to(&next, t))
            }
            _ => None,
        }
    }
}

/// Find
impl<T: Tween,R> Spline<T,R> {
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
    pub fn pair(&self, time: f32) -> (Option<&KeyPoint<T,R>>, Option<&KeyPoint<T,R>>) {
        match self.keypoint_at(time) {
            Ok(index) => (self.points.get(index), self.points.get(index + 1)),
            Err(index) => {
                if index == 0 {
                    (None, self.points.first())
                } else {
                    (self.points.last(), None)
                }
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
            p.time
                .partial_cmp(&time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if val == 0 || val == self.points.len() {
            Err(val)
        } else {
            // SAFETY: val != 0
            let ret_value = val.wrapping_add_signed(-1);
            Ok(ret_value)
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

impl<T: Tween, R> Default for Spline<T, R> {
    fn default() -> Self {
        Self { points: vec![] }
    }
}

/// # Mutations
impl<T: Tween,R> Spline<T,R> {
    pub fn points(&self) -> &Vec<KeyPoint<T,R>> {
        &self.points
    }
    pub fn len(&self) -> usize {
        self.points.len()
    }
    pub fn push(&mut self, keypoint: KeyPoint<T,R>) {
        self.points.push(keypoint);
        self.sort_unstable();
    }

    pub fn remove(&mut self, index: usize) -> Option<KeyPoint<T,R>> {
        if index < self.points.len() {
            Some(self.points.remove(index))
        } else {
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
    pub fn push_many(&mut self, keypoints: impl IntoIterator<Item = KeyPoint<T,R>>) {
        let iter = keypoints.into_iter();
        for keypoint in iter {
            self.points.push(keypoint);
        }
        self.sort_unstable();
    }
    pub fn iter(&self) -> impl Iterator<Item = &KeyPoint<T,R>> {
        self.points.iter()
    }
}
impl<T: Tween, R> From<Vec<KeyPoint<T, R>>> for Spline<T, R> {
    fn from(value: Vec<KeyPoint<T,R>>) -> Self {
        let mut ret = Self { points: value };
        ret.sort_unstable();
        ret
    }
}

impl<T: Tween, R> FromIterator<KeyPoint<T,R>> for Spline<T,R> {
    fn from_iter<I: IntoIterator<Item = KeyPoint<T, R>>>(iter: I) -> Self {
        let mut ret: Self = Default::default();
        ret.push_many(iter);
        ret
    }
}
pub trait Tween: Clone {
    fn lerp(x1: Self, x2: Self, t: f32) -> Self;
    fn ease(x1: Self, x2: Self, t: f32, easing: EasingId) -> Self {
        Self::lerp(x1, x2, easef32(easing, t))
    }
}

impl Tween for f32 {
    fn lerp(x1: Self, x2: Self, t: f32) -> Self {
        t * (x2 - x1) + x1
    }
}

// Jump between values.
impl Tween for usize {
    fn lerp(x1: Self, _x2: Self, _t: f32) -> Self {
        x1
    }
}

pub type Easing = fn(f32) -> f32;

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

#[derive(IntoPrimitive,TryFromPrimitive, Clone, Copy, Debug,Default, PartialEq, Eq)]
#[repr(u8)]
pub enum EasingId {
    #[default]
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    QuadIn,
    QuadOut,
    QuadInOut,
    QubicIn,
    QubicOut,
    QubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    Start,
    End,
    AnimCurve
}

fn easef32(ease_type: EasingId, x: f32) -> f32 {
    let id_raw: u8 = ease_type.into();
    match EASING_MAP.get(id_raw as usize) {
        Some(func) => func(x),
        None => {
            error!("Unknown ease type {:?}", ease_type);
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
        assert_eq!(f32::lerp(0.2, 1.2, 0.9), 1.2);
    }
}
