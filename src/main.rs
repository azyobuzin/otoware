#![feature(min_const_fn)]
#![allow(dead_code)]

#[macro_use]
extern crate bitflags;
extern crate widestring;
extern crate winapi;

mod coreaudio;
mod com_support;

use self::com_support::*;
use self::coreaudio::mmdevice::*;

fn main() {
    spawn_mta_thread(run_on_thread).join().unwrap().unwrap();
}

fn run_on_thread() -> Result<(), Box<std::error::Error + Send + Sync + 'static>> {
    println!("Default Render Endpoint: {:?}", get_default_audio_render_endpoint(Role::Console));
    println!("Default Capture Endpoint: {:?}", get_default_audio_capture_endpoint(Role::Console));

    let endpoints = enumerate_audio_endpoints(DataFlow::All, DeviceState::ACTIVE)?;

    for endpoint in endpoints.into_iter() {
        fn to_string(result: ComResult<widestring::WideCString>) -> Result<String, Box<std::error::Error>> {
            Ok(result?.to_string()?)
        }

        /*
        print!(
            concat!(
                "DeviceInterface FriendlyName: {:?}\n",
                "Device Description: {:?}\n",
                "Device FriendlyName: {:?}\n\n"
            ),
            to_string(endpoint.get_device_interface_friendly_name()),
            to_string(endpoint.get_device_description()),
            to_string(endpoint.get_device_friendly_name())
        );
        */
        println!("{:?}", endpoint);
    }

    Ok(())
}
