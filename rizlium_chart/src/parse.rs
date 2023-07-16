use std::{error::Error, fmt::Display};

pub mod official;
#[derive(Debug)]
pub struct ConvertError(&'static str);
impl Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for ConvertError {}
