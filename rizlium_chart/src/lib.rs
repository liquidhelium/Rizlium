//! # Rizlium 谱面格式
//! Rizlium 所使用的谱面结构及部分操作.

/// 核心数据结构.
pub mod chart;
/// 从文件创建Rizlium谱面.
pub mod parse;

/// 在某个时间的谱面状态.
#[cfg(feature = "runtime")]
pub mod runtime;

#[cfg(feature = "editing")]
pub mod editing;

/// 正常情况下游戏画面截取的部分.
pub const VIEW_RECT: [[f32; 2]; 2] = [[-450., 0.], [450., 1600.]];

pub mod prelude {
    pub use super::chart::*;
    #[cfg(feature = "rizline")]
    pub use super::parse::rizline::RizlineChart;
    #[cfg(feature = "runtime")]
    pub use super::runtime::*;
}

pub mod test_resources {
    use crate::prelude::{Chart, RizlineChart};
    use serde_json::from_str;
    const CHART_TEXT: &str = "todo: this chart was removed, but this test isn't";
    #[static_init::dynamic]
    pub static CHART: Chart = from_str::<RizlineChart>(CHART_TEXT)
        .unwrap()
        .try_into()
        .unwrap();
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::prelude::Chart;

    use super::test_resources::CHART;

    #[test]
    fn serde_chart() {
        serde_json::to_writer_pretty(fs::File::create("./chart-out.json").unwrap(), &*CHART).unwrap();
        let _chart:Chart = serde_json::from_reader(fs::File::open("./chart-out.json").unwrap()).unwrap();
    }
}
