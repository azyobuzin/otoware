#![allow(dead_code)]

extern crate cpal;
extern crate rodio;

mod effects;

use rodio::buffer::SamplesBuffer;
use rodio::Source;

fn main() {
    let input_device = cpal::devices()
        .find(|dev| dev.name() == "スピーカー (Realtek High Definition Audio)")
        .unwrap();
    let output_device = rodio::devices()
        .find(|dev| dev.name() == "Realtek HD Audio 2nd output (Realtek High Definition Audio)")
        .unwrap();

    let mut sink = rodio::Sink::new(&output_device);
    //sink.set_volume(100f32);
    sink.play();

    let event_loop = cpal::EventLoop::new();
    let input_format = input_device.default_input_format().unwrap();
    let stream_id = event_loop.build_input_stream(&input_device, &input_format).unwrap();
    event_loop.play_stream(stream_id);

    event_loop.run(move |_, data| {
        let mut samples =
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

        const FACTOR: f32 = 150.0;
        let buf1 = SamplesBuffer::new(input_format.channels, input_format.sample_rate.0, samples.clone());
        let buf2 = SamplesBuffer::new(input_format.channels, input_format.sample_rate.0, samples.clone());

        
        let source = /*buf1.amplify(0.1)
            .mix(*/
                effects::clipping_amplify(buf2/*.band_pass(4000, 2.0)*/, FACTOR)
                    .amplify(0.7)
            /*)*/;
        
        //let source = effects::clipping_amplify(buf2, FACTOR);

        sink.append(source);
    });
}
