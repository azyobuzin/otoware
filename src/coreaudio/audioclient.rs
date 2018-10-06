use std::mem;
use winapi::shared::basetsd::UINT32;
use winapi::shared::minwindef::DWORD;
use winapi::um::audiosessiontypes::*;
use winapi::um::audioclient::*;
use crate::com_support::*;

#[derive(Debug, Clone)]
pub struct AudioClient(SafeUnknown<IAudioClient>);

// TODO: AUDCLNT_E_DEVICE_INVALIDATED は特別扱いできると良さそう

impl AudioClient {
    pub fn get_buffer_size(&self) -> ComResult<u32> {
        unsafe {
            let mut buffer_frames: UINT32 = mem::uninitialized();
            self.0.GetBufferSize(&mut buffer_frames).to_result()?;
            Ok(buffer_frames as u32)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AudioClientShareMode {
    Shared,
    Exclusive,
}

impl AudioClientShareMode {
    fn to_int(&self) -> AUDCLNT_SHAREMODE {
        match *self {
            AudioClientShareMode::Shared => AUDCLNT_SHAREMODE_SHARED,
            AudioClientShareMode::Exclusive => AUDCLNT_SHAREMODE_EXCLUSIVE,
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct StreamFlags: DWORD {
        const STREAM_CROSSPROCESS = AUDCLNT_STREAMFLAGS_CROSSPROCESS;
        const STREAM_LOOPBACK = AUDCLNT_STREAMFLAGS_LOOPBACK;
        const STREAM_EVENTCALLBACK = AUDCLNT_STREAMFLAGS_EVENTCALLBACK;
        const STREAM_NOPERSIST = AUDCLNT_STREAMFLAGS_NOPERSIST;
        const STREAM_RATEADJUST = AUDCLNT_STREAMFLAGS_RATEADJUST;
        const STREAM_PREVENT_LOOPBACK_CAPTURE = AUDCLNT_STREAMFLAGS_PREVENT_LOOPBACK_CAPTURE;
        const STREAM_AUTOCONVERTPCM = AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM;
        const STREAM_SRC_DEFAULT_QUALITY = AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY;
        const SESSION_EXPIREWHENUNOWNED = AUDCLNT_SESSIONFLAGS_EXPIREWHENUNOWNED;
        const SESSION_DISPLAY_HIDE = AUDCLNT_SESSIONFLAGS_DISPLAY_HIDE;
        const SESSION_DISPLAY_HIDEWHENEXPIRED = AUDCLNT_SESSIONFLAGS_DISPLAY_HIDEWHENEXPIRED;
    }
}
