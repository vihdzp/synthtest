use std::marker::PhantomData;

use rand::{
    distributions::{DistIter, Uniform},
    prelude::{Distribution, ThreadRng},
};

use crate::wav;

pub trait Instrument {
    /// Gets the current audio sample in a single channel.
    fn get_sample_mono<T: wav::AudioSample>(&mut self, time: f64) -> Option<T>;

    /// Gets the current audio sample in all channels.
    fn get_sample<T: wav::AudioSample, const N: usize>(&mut self, time: f64) -> Option<[T; N]> {
        Some([self.get_sample_mono(time)?; N])
    }

    fn iter_mono<'a, T: wav::AudioSample>(
        &'a mut self,
        sample_rate: u32,
    ) -> InstrumentIterMono<'a, T, Self>
    where
        Self: Sized,
    {
        InstrumentIterMono::new(self, sample_rate)
    }

    fn iter<'a, T: wav::AudioSample, const N: usize>(
        &'a mut self,
        sample_rate: u32,
    ) -> InstrumentIter<'a, T, Self, N>
    where
        Self: Sized,
    {
        InstrumentIter::new(self, sample_rate)
    }
}

/// An iterator that continuously gets mono samples from an instrument.
///
/// This iterator will go on forever, unless the instrument itself stops
/// producing samples. Use `take` if you want a fixed amount of samples.
pub struct InstrumentIterMono<'a, T, U> {
    instrument: &'a mut U,
    time: f64,
    tick: f64,
    _phantom: PhantomData<T>,
}

impl<'a, T: wav::AudioSample, U: Instrument> InstrumentIterMono<'a, T, U> {
    pub fn new_with_time(instrument: &'a mut U, time: f64, sample_rate: u32) -> Self {
        Self {
            instrument,
            time,
            tick: 1.0 / sample_rate as f64,
            _phantom: PhantomData,
        }
    }

    pub fn new(instrument: &'a mut U, sample_rate: u32) -> Self {
        Self::new_with_time(instrument, 0.0, sample_rate)
    }
}

impl<'a, T: wav::AudioSample, U: Instrument> Iterator for InstrumentIterMono<'a, T, U> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.time += self.tick;
        self.instrument.get_sample_mono(self.time)
    }
}

/// An iterator that continuously gets samples from an instrument.
///
/// This iterator will go on forever, unless the instrument itself stops
/// producing samples. Use `take` if you want a fixed amount of samples.
pub struct InstrumentIter<'a, T, U, const N: usize>(InstrumentIterMono<'a, T, U>);

impl<'a, T: wav::AudioSample, U: Instrument, const N: usize> InstrumentIter<'a, T, U, N> {
    pub fn new_with_time(instrument: &'a mut U, time: f64, sample_rate: u32) -> Self {
        Self(InstrumentIterMono::new_with_time(
            instrument,
            time,
            sample_rate,
        ))
    }

    pub fn new(instrument: &'a mut U, sample_rate: u32) -> Self {
        Self::new_with_time(instrument, 0.0, sample_rate)
    }
}

impl<'a, T: wav::AudioSample, U: Instrument, const N: usize> Iterator
    for InstrumentIter<'a, T, U, N>
{
    type Item = [T; N];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.time += self.0.tick;
        self.0.instrument.get_sample(self.0.time)
    }
}

/// Represents a basic square wave.
pub struct Square {
    /// The frequency of the wave in Hertz.
    freq: f64,
}

impl Default for Square {
    fn default() -> Self {
        Self { freq: 440.0 }
    }
}

impl Square {
    pub fn new(freq: f64) -> Self {
        Self { freq }
    }
}

impl Instrument for Square {
    fn get_sample_mono<T: wav::AudioSample>(&mut self, time: f64) -> Option<T> {
        Some(if time * self.freq % 1.0 < 0.5 {
            T::MIN
        } else {
            T::MAX
        })
    }
}

/// Represents a basic saw wave.
pub struct Saw {
    /// The frequency of the wave in Hertz.
    freq: f64,
}

impl Default for Saw {
    fn default() -> Self {
        Self { freq: 440.0 }
    }
}

impl Saw {
    pub fn new(freq: f64) -> Self {
        Self { freq }
    }
}

impl Instrument for Saw {
    fn get_sample_mono<T: wav::AudioSample>(&mut self, time: f64) -> Option<T> {
        Some(T::from_f64(time * self.freq % 1.0))
    }
}

/// Represents white noise.
pub struct Random(DistIter<Uniform<f64>, ThreadRng, f64>);

impl Default for Random {
    fn default() -> Self {
        Self(Uniform::new(0.0, 1.0).sample_iter(rand::thread_rng()))
    }
}

impl Random {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Instrument for Random {
    fn get_sample_mono<T: wav::AudioSample>(&mut self, _: f64) -> Option<T> {
        Some(T::from_f64(self.0.next()?))
    }
}
