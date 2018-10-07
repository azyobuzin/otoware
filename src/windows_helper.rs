extern crate winapi;

use std::io;
use std::mem;
use iui::controls;
use iui::prelude::*;
use log;
use ui_sys;
use self::winapi::shared::windef::HWND;
use self::winapi::um::*;
use self::winapi::um::winuser::*;

pub fn make_push_like(checkbox: &mut controls::Checkbox) {
    unsafe {
        let hwnd = get_hwnd(&checkbox.clone().into());

        let current_style = {
            errhandlingapi::SetLastError(0);
            let result = GetWindowLongPtrW(hwnd, GWL_STYLE);

            if result == 0 {
                let err = errhandlingapi::GetLastError();
                if err != 0 {
                    error!("GetWindowLongPtrW failed in make_push_like: {}", io::Error::from_raw_os_error(err as i32));
                    return;
                }
            }
            result
        };

        // スタイルに BS_PUSHLIKE を追加
        let new_style = current_style | BS_PUSHLIKE as isize;

        errhandlingapi::SetLastError(0);
        let result = SetWindowLongPtrW(hwnd, GWL_STYLE, new_style);

        if result == 0 && log_enabled!(log::Level::Error) {
            let err = errhandlingapi::GetLastError();
            if err != 0 {
                error!("SetWindowLongPtrW failed in make_push_like: {}", io::Error::from_raw_os_error(err as i32));
            }
        }
    }
    // TODO: MinimumSize を書き換えて Button のものにしたらサイズ感もよくなりそうだけど
}

unsafe fn get_hwnd(control: &controls::Control) -> HWND {
    let control_ptr = control.as_ui_control() as usize;
    let hwnd_ptr = control_ptr
        + mem::size_of::<ui_sys::platform::windows::uiWindowsControl>()
        + mem::size_of::<usize>(); // ui-sys には含まれていないもう 1 つの関数ポインタフィールドがある
    (hwnd_ptr as *const HWND).read()
}
