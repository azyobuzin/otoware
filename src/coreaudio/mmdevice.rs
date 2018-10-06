use std::fmt;
use std::mem;
use std::ptr::{self, null_mut};
use widestring::WideCString;
use winapi::Interface;
use winapi::ctypes::c_ushort;
use winapi::shared::devpkey;
use winapi::shared::devpropdef::DEVPROPKEY;
use winapi::shared::minwindef::{DWORD, LPVOID, UINT};
use winapi::shared::wtypes::*;
use winapi::um::*;
use winapi::um::mmdeviceapi::*;
use winapi::um::winnt::LPWSTR;
use crate::com_support::*;

fn create_enumerator() -> ComResult<SafeUnknown<IMMDeviceEnumerator>> {
    unsafe {
        let mut enumerator = NullableSafeUnknown::null();
        combaseapi::CoCreateInstance(
            &CLSID_MMDeviceEnumerator,
            null_mut(),
            combaseapi::CLSCTX_ALL,
            &IMMDeviceEnumerator::uuidof(),
            enumerator.as_void_ref_ptr()
        ).to_result()?;
        Ok(enumerator.not_null())
    }
}

fn get_default_audio_endpoint(data_flow: DataFlow, role: Role) -> ComResult<Option<MMDevice>> {
    let enumerator = create_enumerator()?;
    let mut endpoint = NullableSafeUnknown::null();

    let result = unsafe {
        enumerator.GetDefaultAudioEndpoint(
            data_flow.to_edataflow(),
            role.to_erole(),
            endpoint.as_mut_ref_ptr()
        ).to_result()
    };

    match result {
        Ok(_) => Ok(Some(MMDevice(unsafe { endpoint.not_null() }))),
        Err(ComError(E_NOTFOUND)) => Ok(None),
        Err(x) => Err(x),
    }
}

pub fn get_default_audio_render_endpoint(role: Role) -> ComResult<Option<MMDevice>> {
    get_default_audio_endpoint(DataFlow::Render, role)
}

pub fn get_default_audio_capture_endpoint(role: Role) -> ComResult<Option<MMDevice>> {
    get_default_audio_endpoint(DataFlow::Capture, role)
}

pub fn enumerate_audio_endpoints(data_flow: DataFlow, state_mask: DeviceStateMask) -> ComResult<Vec<MMDevice>> {
    if state_mask.is_empty() { return Ok(Vec::default()); }

    let enumerator = create_enumerator()?;

    let collection = unsafe {
        let mut collection = NullableSafeUnknown::null();
        enumerator.EnumAudioEndpoints(
            data_flow.to_edataflow(),
            state_mask.bits(),
            collection.as_mut_ref_ptr()
        ).to_result()?;
        collection.not_null()
    };

    let count = unsafe {
        let mut count: UINT = 0;
        // GetCount の定義が *const UINT になっているが、実際には変更される
        collection.GetCount(&mut count as *mut UINT as *const UINT).to_result()?;
        count
    };

    let mut result = Vec::with_capacity(count as usize);
    for i in 0..count {
        unsafe {
            let mut device = NullableSafeUnknown::null();
            collection.Item(i, device.as_mut_ref_ptr()).to_result()?;
            result.push(MMDevice(device.not_null()));
        }
    }

    Ok(result)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DataFlow {
    Render,
    Capture,
    All,
}

impl DataFlow {
    fn to_edataflow(&self) -> EDataFlow {
        match *self {
            DataFlow::Render => eRender,
            DataFlow::Capture => eCapture,
            DataFlow::All => eAll,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Role {
    Console,
    Multimedia,
    Communications,
}

impl Role {
    fn to_erole(&self) -> ERole {
        match *self {
            Role::Console => eConsole,
            Role::Multimedia => eMultimedia,
            Role::Communications => eCommunications,
        }
    }
}

bitflags! {
    pub struct DeviceStateMask: DWORD {
        const ACTIVE = 0x00000001;
        const DISABLED = 0x00000002;
        const NOTPRESENT = 0x00000004;
        const UNPLUGGED = 0x00000008;
        const ALL = 0x0000000f;
    }
}

#[derive(Clone)]
pub struct MMDevice(SafeUnknown<IMMDevice>);

impl MMDevice {
    pub fn get_id(&self) -> ComResult<WideCString> {
        unsafe {
            let mut pstr_id = null_mut();
            self.0.GetId(&mut pstr_id).to_result()?;

            let str_id = WideCString::from_ptr_str(pstr_id);

            combaseapi::CoTaskMemFree(pstr_id as LPVOID);

            Ok(str_id)
        }
    }

    // TODO: GetState

    pub fn get_device_interface_friendly_name(&self) -> ComResult<WideCString> {
        get_string_property_with_devpkey(self, &devpkey::DEVPKEY_DeviceInterface_FriendlyName)
    }

    pub fn get_device_description(&self) -> ComResult<WideCString> {
        get_string_property_with_devpkey(self, &devpkey::DEVPKEY_Device_DeviceDesc)
    }

    pub fn get_device_friendly_name(&self) -> ComResult<WideCString> {
        get_string_property_with_devpkey(self, &devpkey::DEVPKEY_Device_FriendlyName)
    }
}

impl fmt::Debug for MMDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ds = f.debug_struct("MMDevice");
        ds.field("0", &self.0);

        if let Ok(cs) = self.get_id() {
            ds.field("id", &cs.to_os_string());
        }

        if let Ok(cs) = self.get_device_friendly_name() {
            ds.field("device_friendly_name", &cs.to_os_string());
        }

        ds.finish()
    }
}

fn get_string_property(device: &MMDevice, key: &PROPERTYKEY) -> ComResult<WideCString> {
    let property_store = unsafe {
        let mut property_store = NullableSafeUnknown::null();
        device.0.OpenPropertyStore(
            coml2api::STGM_READ,
            property_store.as_mut_ref_ptr()
        ).to_result()?;
        property_store.not_null()
    };

    let propvar = unsafe {
        let mut propvar = mem::uninitialized();
        property_store.GetValue(key as propkeydef::REFPROPERTYKEY, &mut propvar).to_result()?;
        propvar
    };

    assert_eq!(propvar.vt, VT_LPWSTR as c_ushort);

    unsafe {
        let value_ptr = ptr::read(&propvar.data as *const [u8] as *const LPWSTR);
        Ok(WideCString::from_ptr_str(value_ptr))
    }

    // TODO: この文字列のメモリ解放ってどうするの？
}

fn get_string_property_with_devpkey(device: &MMDevice, key: &DEVPROPKEY) -> ComResult<WideCString> {
    get_string_property(device, unsafe { mem::transmute(key) })
}
