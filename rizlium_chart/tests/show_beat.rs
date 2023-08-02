use rizlium_chart::prelude::*;

#[test]
fn main() {
    let chart = rizlium_chart::__test_chart();
    let cache = ChartCache::from_chart(&chart);
    std::fs::write(
        "/home/helium/code/rizlium/rizlium_chart/tests/beats",
        format!("{:#?}", cache.beat),
    )
    .unwrap();
}
