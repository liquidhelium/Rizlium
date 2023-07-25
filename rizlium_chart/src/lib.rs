

pub mod chart;
pub mod parse;
#[cfg(feature="runtime")]
pub mod runtime;
pub const VIEW_RECT: [[f32; 2]; 2] = [[-450., 0.], [450., 1600.]];

pub fn __test_chart() -> chart::Chart {
    serde_json::from_str::<parse::official_next::RizlineChart>(include_str!("../test_assets/take.json"))
        .unwrap()
        .try_into().unwrap()
}
