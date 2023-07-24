use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature="atomic")] {

        type Refc<T> = std::sync::Arc<T>;
        type Weak<T> = std::sync::Weak<T>;
    }
    else {
        type Refc<T> = std::rc::Rc<T>;
        type Weak<T> = std::rc::Weak<T>;

    }
}
pub mod chart;
pub mod parse;
#[cfg(feature="runtime")]
pub mod runtime;
pub const VIEW_RECT: [[f32; 2]; 2] = [[-450., 0.], [450., 1600.]];

pub fn __test_chart() -> chart::ChartNext {
    serde_json::from_str::<parse::official_next::RizlineChart>(include_str!("../test_assets/take.json"))
        .unwrap()
        .into()
}
