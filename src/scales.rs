pub trait Scale {
    /// Converts a note to a frequency. By convention, note 0 is tuned to 27.5
    /// Hz.
    fn to_freq(self, note: i16) -> f64;
}

/// Equal Division of the Octave into a `x` intervals.
///
/// The single value of this field is `2 ^ (1 / x)`.
pub struct Edo(f64);

impl Edo {
    pub fn new(x: f64) -> Self {
        Self(f64::powf(2.0, 1.0 / x))
    }
}

impl Scale for Edo {
    fn to_freq(self, note: i16) -> f64 {
        self.0.powf(note as f64)
    }
}
