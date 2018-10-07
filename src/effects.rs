use std::time::Duration;
use rodio::Sample;
use rodio::Source;

pub fn clipping_amplify<I>(input: I, factor: f32) -> ClippingAmplify<I>
    where I: Source, I::Item: Sample
{
    ClippingAmplify { input, factor }
}

#[derive(Clone, Debug)]
pub struct ClippingAmplify<I> {
    input: I,
    factor: f32,
}

impl<I> Iterator for ClippingAmplify<I>
    where I: Source, I::Item: Sample
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.next()
            .map(|x| (cpal::Sample::to_f32(&x) * self.factor).min(1.0).max(-1.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I> ExactSizeIterator for ClippingAmplify<I>
    where I: Source + ExactSizeIterator, I::Item: Sample
{
}

impl<I> Source for ClippingAmplify<I>
    where I: Source, I::Item: Sample
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.input.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
