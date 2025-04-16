pub mod algo;
use algo::*;

/// A two-dimensional point on screen
#[derive(Debug, Default)]
pub struct Point {
    pub v: i16,
    pub h: i16,
}
