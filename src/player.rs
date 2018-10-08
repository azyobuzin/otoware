use std::sync;
use std::sync::atomic;
use std::thread;
use cpal;
use rodio;
use rodio::Source;
use super::effects::clipping_amplify;

pub struct OtowarePlayer {
    inner: sync::Arc<OtowarePlayerInner>,
}

struct OtowarePlayerInner {
    event_loop: cpal::EventLoop,
    input: sync::RwLock<Option<(cpal::StreamId, cpal::Format)>>,
    output_sink: sync::RwLock<Option<rodio::Sink>>,
    gain: atomic::AtomicUsize, // 0 - 100 [dB]
    volume: atomic::AtomicUsize, // 0 - 100 [%]
    dropped: atomic::AtomicBool,
}

impl OtowarePlayer {
    pub fn new() -> OtowarePlayer {
        let instance = OtowarePlayer {
            inner: sync::Arc::new(
                OtowarePlayerInner {
                    event_loop: cpal::EventLoop::new(),
                    input: Default::default(),
                    output_sink: Default::default(),
                    gain: atomic::AtomicUsize::new(0),
                    volume: atomic::AtomicUsize::new(50),
                    dropped: atomic::AtomicBool::new(false),
                }
            ),
        };

        spawn_event_loop(instance.inner.clone());

        instance
    }

    pub fn set_input(&mut self, device: &cpal::Device) -> Result<(), cpal::DefaultFormatError> {
        let event_loop = &self.inner.event_loop;
        let format = device.default_input_format()?;
        let stream_id = event_loop.build_input_stream(device, &format)
            .map_err(|e| match e {
                cpal::CreationError::DeviceNotAvailable => cpal::DefaultFormatError::DeviceNotAvailable,
                cpal::CreationError::FormatNotSupported => panic!("default_input_format がおかしい")
            })?;

        {
            let mut lock = self.inner.input.write().unwrap();
            let input = &mut *lock;
            if let Some((prev_stream, _)) = input.take() {
                event_loop.destroy_stream(prev_stream);
            }

            *input = Some((stream_id.clone(), format));
        }

        event_loop.play_stream(stream_id);
        Ok(())
    }

    pub fn set_output(&mut self, device: &rodio::Device) {
        let sink = rodio::Sink::new(device);
        sink.play();

        // 前の Sink を drop しつつ、新しい Sink を設定
        *self.inner.output_sink.write().unwrap() = Some(sink);
    }

    pub fn clear(&mut self) {
        // input を None に
        let mut input = self.inner.input.write().unwrap();
        if let Some((prev_stream, _)) = input.take() {
            self.inner.event_loop.destroy_stream(prev_stream);
        }

        // output を None に
        *self.inner.output_sink.write().unwrap() = None;
    }

    pub fn set_gain(&mut self, value: u8) {
        self.inner.gain.store(value as usize, atomic::Ordering::Relaxed);
    }

    pub fn set_volume(&mut self, value: u8) {
        assert!(value <= 100);
        self.inner.volume.store(value as usize, atomic::Ordering::Relaxed);
    }
}

impl Drop for OtowarePlayer {
    fn drop(&mut self) {
        self.inner.dropped.store(true, atomic::Ordering::Relaxed);
    }
}

fn spawn_event_loop(inner: sync::Arc<OtowarePlayerInner>) {
    use std::panic;

    thread::spawn(move || {
        let result = panic::catch_unwind(|| {
            inner.event_loop.run(|stream_id, data| {
                if inner.output_sink.read().unwrap().is_none() {
                    return;
                }

                let (channels, sample_rate) = {
                    let lock = inner.input.read().unwrap();
                    match *lock {
                        Some((ref input_stream_id, ref format)) => {
                            if stream_id != *input_stream_id { return; }
                            (format.channels, format.sample_rate.0)
                        }
                        None => return
                    }
                };

                let samples =
                    match data {
                        cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::U16(buffer) } => {
                            buffer.iter().map(cpal::Sample::to_f32).collect()
                        }
                        cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::I16(buffer) } => {
                            buffer.iter().map(cpal::Sample::to_f32).collect()
                        }
                        cpal::StreamData::Input { buffer: cpal::UnknownTypeInputBuffer::F32(buffer) } => {
                            buffer.to_vec()
                        }
                        cpal::StreamData::Output { buffer: _ } => unreachable!()
                    };

                let buf = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);

                let factor = 10.0f32.powf(inner.gain.load(atomic::Ordering::Relaxed) as f32 / 20.0);
                let source = clipping_amplify(buf, factor)
                    .amplify(inner.volume.load(atomic::Ordering::Relaxed) as f32 / 100.0);

                let lock = inner.output_sink.read().unwrap();
                if let Some(ref sink) = *lock {
                    sink.append(source);
                }
            });
        });

        if inner.dropped.load(atomic::Ordering::Relaxed) {
            return;
        }

        // drop されていないのに panic 起こされたらどうにもならない
        if let Err(err) = result {
            panic::resume_unwind(err);
        }
    });
}
