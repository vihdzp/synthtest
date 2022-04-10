use std::{
    fmt,
    fs::File,
    io::{self, Write},
    mem, ops,
    path::Path,
};

/// One of the allowed primitive types for an audio file. This determines the
/// bits per sample.
pub trait AudioSample: fmt::Debug + Copy + ops::Add<Self> + ops::AddAssign<Self> {
    /// The value representing no sound.
    const ZERO: Self;

    /// The minimum value for the type.
    const MIN: Self;

    /// The maximum value for the type.
    const MAX: Self;

    /// Converts a `f64` in the range [0, 1] to this type.
    fn from_f64(x: f64) -> Self;

    /// Converts the given numerical type to little endian.
    fn to_le_bytes(self) -> Vec<u8>;
}

impl AudioSample for u8 {
    const ZERO: u8 = 128;
    const MIN: u8 = u8::MIN;
    const MAX: u8 = u8::MAX;

    fn from_f64(x: f64) -> Self {
        (Self::MAX as f64 * x) as Self
    }

    fn to_le_bytes(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl AudioSample for i16 {
    const ZERO: i16 = 0;
    const MIN: i16 = i16::MIN;
    const MAX: i16 = i16::MAX;

    fn from_f64(x: f64) -> Self {
        (Self::MAX as f64 * x) as Self
    }

    fn to_le_bytes(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl AudioSample for i32 {
    const ZERO: i32 = 0;
    const MIN: i32 = i32::MIN;
    const MAX: i32 = i32::MAX;

    fn from_f64(x: f64) -> Self {
        (Self::MAX as f64 * x) as Self
    }

    fn to_le_bytes(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

/// Represents the raw bytes in an audio stream, in the same layout as the WAV
/// file, as well as associated metadata.
///
/// The number of channels is given by `N`.
pub struct AudioData<T: AudioSample, const N: usize> {
    /// The raw bytes in the audio stream.
    data: Vec<[T; N]>,

    /// Number of samples per second.
    sample_rate: u32,
}

impl<T: AudioSample, const N: usize> AudioData<T, N> {
    /// Initializes empty audio data.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            data: Vec::new(),
            sample_rate,
        }
    }

    /// Resizes the buffer with new empty data.
    fn resize(&mut self, sample_count: u32) {
        if self.data.len() < sample_count as usize {
            self.data.resize(sample_count as usize, [T::ZERO; N])
        }
    }

    /// Performs an action for each entry of the data vector, starting from a
    /// given sample. Extends the vector with the zero value if needed.
    fn do_data<U, I: Iterator<Item = [T; N]>>(
        &mut self,
        init_sample: u32,
        mut iter: I,
        f: fn(&mut [T; N], [T; N]) -> U,
    ) {
        self.resize(init_sample);
        for i in init_sample..self.data.len() as u32 {
            print!("Check 3");
            if let Some(t) = iter.next() {
                f(&mut self.data[i as usize], t);
            } else {
                return;
            }
        }

        print!("Check 1");

        while let Some(t) = iter.next() {
            self.data.push([T::ZERO; N]);
            f(self.data.last_mut().unwrap(), t);
        }
        print!("Check 2");
    }

    /// Writes a raw data sample at the end of the audio data.
    pub fn push_data(&mut self, sample: [T; N]) {
        self.data.push(sample)
    }

    /// Writes a raw data buffer at the end of the audio data.
    pub fn extend_data<I: Iterator<Item = [T; N]>>(&mut self, iter: I) {
        self.data.extend(iter)
    }

    /// Writes a raw data buffer at a given sample. This overwrites anything
    /// previously written.
    pub fn write_data_at<I: Iterator<Item = [T; N]>>(&mut self, init_sample: u32, iter: I) {
        self.do_data(init_sample, iter, |x, y| *x = y)
    }

    /// Writes a raw data buffer at a given sample. This is added to anything
    /// previously written. This will write over all channels at the same time.
    pub fn add_data_at<I: Iterator<Item = [T; N]>>(&mut self, init_sample: u32, iter: I) {
        self.do_data(init_sample, iter, |x, y| {
            for (i, &j) in x.iter_mut().zip(&y) {
                *i = (*i).saturating_add(j)
            }
        })
    }

    /// Number of bytes per second.
    fn byte_rate(&self) -> u32 {
        self.sample_rate * N as u32 * mem::size_of::<T>() as u32
    }

    /// Number of bytes per sample.
    fn block_align(&self) -> u16 {
        N as u16 * mem::size_of::<T>() as u16
    }

    /// Saves the audio data to a WAV file.
    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        let size = self.data.len() as u32 * mem::size_of::<T>() as u32 * N as u32;
        let mut file = File::create(path)?;

        // Writes header.
        file.write_all(b"RIFF")?;
        file.write_all(&(36 + size).to_le_bytes())?;
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&[16, 0, 0, 0, 1, 0])?;
        file.write_all(&(N as u16).to_le_bytes())?;
        file.write_all(&self.sample_rate.to_le_bytes())?;
        file.write_all(&self.byte_rate().to_le_bytes())?;
        file.write_all(&self.block_align().to_le_bytes())?;
        file.write_all(&(mem::size_of::<T>() as u16 * 8).to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&size.to_le_bytes())?;

        // Writes the audio data.
        for sample in &self.data {
            for channel in sample {
                file.write_all(&channel.to_le_bytes())?;
            }
        }
        Ok(())
    }
}
