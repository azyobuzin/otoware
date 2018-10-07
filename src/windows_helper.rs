extern crate ui_sys;
extern crate winapi;

use std::mem;
use iui::controls;
use iui::prelude::*;
use self::winapi::shared::windef::HWND;
use self::winapi::um::winuser::*;

pub fn make_push_like(checkbox: &mut controls::Checkbox) {
    unsafe {
        let hwnd = get_hwnd(&checkbox.clone().into());
        let current_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let new_style = current_style | BS_PUSHLIKE as isize;
        SetWindowLongPtrW(hwnd, GWL_STYLE, new_style);
    }
    // TODO: MinimumSize を書き換えて Button のものにしたらサイズ感もよくなりそうだけど
}

pub unsafe fn get_hwnd(control: &controls::Control) -> HWND {
    let control_ptr = control.as_ui_control() as usize;
    let hwnd_ptr = control_ptr
        + mem::size_of::<ui_sys::platform::windows::uiWindowsControl>()
        + mem::size_of::<usize>(); // ui-sys には含まれていないもう 1 つの関数ポインタフィールドがある
    (hwnd_ptr as *const HWND).read()
}
