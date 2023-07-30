# Diff Details

Date : 2023-07-28 13:29:08

Directory /home/helium/code/rizlium

Total : 69 files,  41149 codes, 104 comments, 514 blanks, all 41767 lines

[Summary](results.md) / [Details](details.md) / [Diff Summary](diff.md) / Diff Details

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [Cargo.lock](/Cargo.lock) | TOML | 3,628 | 2 | 392 | 4,022 |
| [Cargo.toml](/Cargo.toml) | TOML | 11 | 4 | 4 | 19 |
| [rizlium_chart/Cargo.toml](/rizlium_chart/Cargo.toml) | TOML | 15 | 1 | 5 | 21 |
| [rizlium_chart/src/chart.rs](/rizlium_chart/src/chart.rs) | Rust | 178 | 10 | 15 | 203 |
| [rizlium_chart/src/chart/color.rs](/rizlium_chart/src/chart/color.rs) | Rust | 23 | 2 | 4 | 29 |
| [rizlium_chart/src/chart/easing.rs](/rizlium_chart/src/chart/easing.rs) | Rust | 228 | 60 | 18 | 306 |
| [rizlium_chart/src/chart/line.rs](/rizlium_chart/src/chart/line.rs) | Rust | 9 | 1 | 2 | 12 |
| [rizlium_chart/src/chart/note.rs](/rizlium_chart/src/chart/note.rs) | Rust | 16 | 1 | 3 | 20 |
| [rizlium_chart/src/chart/theme.rs](/rizlium_chart/src/chart/theme.rs) | Rust | 30 | 0 | 7 | 37 |
| [rizlium_chart/src/chart/time.rs](/rizlium_chart/src/chart/time.rs) | Rust | 4 | 1 | 2 | 7 |
| [rizlium_chart/src/lib.rs](/rizlium_chart/src/lib.rs) | Rust | 11 | 6 | 5 | 22 |
| [rizlium_chart/src/parse.rs](/rizlium_chart/src/parse.rs) | Rust | 14 | 0 | 4 | 18 |
| [rizlium_chart/src/parse/official.rs](/rizlium_chart/src/parse/official.rs) | Rust | 331 | 3 | 57 | 391 |
| [rizlium_chart/src/runtime.rs](/rizlium_chart/src/runtime.rs) | Rust | 18 | 1 | 3 | 22 |
| [rizlium_chart/test_assets/take.json](/rizlium_chart/test_assets/take.json) | JSON | 36,923 | 0 | 1 | 36,924 |
| [rizlium_render/Cargo.toml](/rizlium_render/Cargo.toml) | TOML | 29 | 2 | 3 | 34 |
| [rizlium_render/src/lib.rs](/rizlium_render/src/lib.rs) | Rust | 123 | 4 | 16 | 143 |
| [rizlium_render/src/line_rendering.rs](/rizlium_render/src/line_rendering.rs) | Rust | 172 | 10 | 22 | 204 |
| [rizlium_render_main/Cargo.toml](/rizlium_render_main/Cargo.toml) | TOML | 6 | 1 | 2 | 9 |
| [rizlium_render_main/src/main.rs](/rizlium_render_main/src/main.rs) | Rust | 3 | 0 | 1 | 4 |
| [c:/code/rust/rizlium/Cargo.toml](/c:/code/rust/rizlium/Cargo.toml) | TOML | -5 | 0 | 0 | -5 |
| [c:/code/rust/rizlium/rizlium_chart/Cargo.toml](/c:/code/rust/rizlium/rizlium_chart/Cargo.toml) | TOML | -11 | -1 | -2 | -14 |
| [c:/code/rust/rizlium/rizlium_chart/src/chart.rs](/c:/code/rust/rizlium/rizlium_chart/src/chart.rs) | Rust | -20 | 0 | -4 | -24 |
| [c:/code/rust/rizlium/rizlium_chart/src/chart/color.rs](/c:/code/rust/rizlium/rizlium_chart/src/chart/color.rs) | Rust | -20 | 0 | -4 | -24 |
| [c:/code/rust/rizlium/rizlium_chart/src/chart/easing.rs](/c:/code/rust/rizlium/rizlium_chart/src/chart/easing.rs) | Rust | -195 | 0 | -17 | -212 |
| [c:/code/rust/rizlium/rizlium_chart/src/chart/line.rs](/c:/code/rust/rizlium/rizlium_chart/src/chart/line.rs) | Rust | -29 | -2 | -2 | -33 |
| [c:/code/rust/rizlium/rizlium_chart/src/chart/note.rs](/c:/code/rust/rizlium/rizlium_chart/src/chart/note.rs) | Rust | -17 | 0 | -3 | -20 |
| [c:/code/rust/rizlium/rizlium_chart/src/lib.rs](/c:/code/rust/rizlium/rizlium_chart/src/lib.rs) | Rust | -2 | 0 | -1 | -3 |
| [c:/code/rust/rizlium/rizlium_chart/src/parse.rs](/c:/code/rust/rizlium/rizlium_chart/src/parse.rs) | Rust | -10 | 0 | -2 | -12 |
| [c:/code/rust/rizlium/rizlium_chart/src/parse/official.rs](/c:/code/rust/rizlium/rizlium_chart/src/parse/official.rs) | Rust | -275 | -2 | -17 | -294 |
| [c:/code/rust/rizlium/target/.rustc_info.json](/c:/code/rust/rizlium/target/.rustc_info.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/itoa-ec301f21199e026c/lib-itoa.json](/c:/code/rust/rizlium/target/debug/.fingerprint/itoa-ec301f21199e026c/lib-itoa.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/log-21a96e16073bfb5e/lib-log.json](/c:/code/rust/rizlium/target/debug/.fingerprint/log-21a96e16073bfb5e/lib-log.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/log-60ff87092fa059c3/lib-log.json](/c:/code/rust/rizlium/target/debug/.fingerprint/log-60ff87092fa059c3/lib-log.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/nutype-06f297ec80e7d4e7/lib-nutype.json](/c:/code/rust/rizlium/target/debug/.fingerprint/nutype-06f297ec80e7d4e7/lib-nutype.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/nutype-c030aa77be35bb98/lib-nutype.json](/c:/code/rust/rizlium/target/debug/.fingerprint/nutype-c030aa77be35bb98/lib-nutype.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/nutype_macros-35105d637b40a10b/lib-nutype_macros.json](/c:/code/rust/rizlium/target/debug/.fingerprint/nutype_macros-35105d637b40a10b/lib-nutype_macros.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/proc-macro2-29a85740baa5d895/lib-proc-macro2.json](/c:/code/rust/rizlium/target/debug/.fingerprint/proc-macro2-29a85740baa5d895/lib-proc-macro2.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/proc-macro2-72f70bc0176d70be/run-build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/proc-macro2-72f70bc0176d70be/run-build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/proc-macro2-9bd74a526cdee8a2/build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/proc-macro2-9bd74a526cdee8a2/build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/quote-5e891bba88709f2b/run-build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/quote-5e891bba88709f2b/run-build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/quote-c6ceed820145882c/build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/quote-c6ceed820145882c/build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/quote-f747001eb23ca389/lib-quote.json](/c:/code/rust/rizlium/target/debug/.fingerprint/quote-f747001eb23ca389/lib-quote.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-04fb442fefb58e16/test-lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-04fb442fefb58e16/test-lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-093aecf67e6b7f53/test-lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-093aecf67e6b7f53/test-lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-1b053262c175a574/lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-1b053262c175a574/lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-450fa17f5dc4a83b/test-lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-450fa17f5dc4a83b/test-lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-644c75374469c20e/test-lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-644c75374469c20e/test-lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-7be9489879a7db56/test-lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-7be9489879a7db56/test-lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-92c2d69ecac4ada0/lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-92c2d69ecac4ada0/lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-a31179ed79e9e962/test-lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-a31179ed79e9e962/test-lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-bc2dbb86df9d32c8/lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-bc2dbb86df9d32c8/lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-e89dd71a0e6c78e6/lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-e89dd71a0e6c78e6/lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-f84c8d32a32b8f79/lib-rizlium_chart.json](/c:/code/rust/rizlium/target/debug/.fingerprint/rizlium_chart-f84c8d32a32b8f79/lib-rizlium_chart.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/ryu-a95b30365a9fc6c0/lib-ryu.json](/c:/code/rust/rizlium/target/debug/.fingerprint/ryu-a95b30365a9fc6c0/lib-ryu.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/serde-33e3531276be84e3/lib-serde.json](/c:/code/rust/rizlium/target/debug/.fingerprint/serde-33e3531276be84e3/lib-serde.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/serde-9045e75fd40b74c7/build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/serde-9045e75fd40b74c7/build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/serde-a940a7e37baaea6e/run-build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/serde-a940a7e37baaea6e/run-build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/serde_derive-b57b9ab09364abd1/lib-serde_derive.json](/c:/code/rust/rizlium/target/debug/.fingerprint/serde_derive-b57b9ab09364abd1/lib-serde_derive.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/serde_json-9d9d8f2092469510/lib-serde_json.json](/c:/code/rust/rizlium/target/debug/.fingerprint/serde_json-9d9d8f2092469510/lib-serde_json.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/serde_json-b4a3b9187e084d31/run-build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/serde_json-b4a3b9187e084d31/run-build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/serde_json-bd050b0904a290af/build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/serde_json-bd050b0904a290af/build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/simple-easing-a03dadf1c0f5110e/lib-simple-easing.json](/c:/code/rust/rizlium/target/debug/.fingerprint/simple-easing-a03dadf1c0f5110e/lib-simple-easing.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/simple-easing-f5df8fef84546625/lib-simple-easing.json](/c:/code/rust/rizlium/target/debug/.fingerprint/simple-easing-f5df8fef84546625/lib-simple-easing.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/syn-18e0d91c0153b3be/build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/syn-18e0d91c0153b3be/build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/syn-1ae70b56e1e65ab1/run-build-script-build-script-build.json](/c:/code/rust/rizlium/target/debug/.fingerprint/syn-1ae70b56e1e65ab1/run-build-script-build-script-build.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/syn-8182f09488e75109/lib-syn.json](/c:/code/rust/rizlium/target/debug/.fingerprint/syn-8182f09488e75109/lib-syn.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/syn-cfe4554c45697191/lib-syn.json](/c:/code/rust/rizlium/target/debug/.fingerprint/syn-cfe4554c45697191/lib-syn.json) | JSON | -1 | 0 | 0 | -1 |
| [c:/code/rust/rizlium/target/debug/.fingerprint/unicode-ident-9e168a72a6fce038/lib-unicode-ident.json](/c:/code/rust/rizlium/target/debug/.fingerprint/unicode-ident-9e168a72a6fce038/lib-unicode-ident.json) | JSON | -1 | 0 | 0 | -1 |

[Summary](results.md) / [Details](details.md) / [Diff Summary](diff.md) / Diff Details