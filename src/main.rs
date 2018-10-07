#![allow(dead_code)]

extern crate cpal;
extern crate env_logger;
extern crate iui;
#[macro_use] extern crate log;
extern crate libc;
extern crate rodio;
extern crate ui_sys;

mod effects;
mod player;
mod windows_helper;

use iui::controls::*;
use iui::prelude::*;

fn main() {
    env_logger::init();

    let ui = UI::init().unwrap();

    // レイアウト
    let mut root_grid = LayoutGrid::new(&ui);

    let (mut input_combo, mut output_combo, mut gain_slider, mut volume_slider) = {
        let ui_clone = ui.clone();
        let mut root_grid_clone = root_grid.clone();
        let mut append_input_control = move |row, label, control: Control| {
            root_grid_clone.append(
                &ui_clone, Label::new(&ui_clone, label),
                0, row, 1, 1,
                GridExpand::Neither,
                GridAlignment::Start,
                GridAlignment::Center
            );
            root_grid_clone.append(
                &ui_clone, control,
                1, row, 1, 1,
                GridExpand::Horizontal,
                GridAlignment::Fill,
                GridAlignment::Center
            );
        };

        let input_combo = Combobox::new(&ui);
        append_input_control(0, "入力: ", input_combo.clone().into());

        let output_combo = Combobox::new(&ui);
        append_input_control(1, "出力: ", output_combo.clone().into());

        let gain_slider = Slider::new(&ui, 0, 80);
        append_input_control(2, "ゲイン: ", gain_slider.clone().into());

        let mut volume_slider = Slider::new(&ui, 0, 100);
        volume_slider.set_value(&ui, 50);
        append_input_control(3, "音量: ", volume_slider.clone().into());

        (input_combo, output_combo, gain_slider, volume_slider)
    };

    let mut running_toggle = Checkbox::new(&ui, "音割れさせる");
    windows_helper::make_push_like(&mut running_toggle);
    root_grid.append(
        &ui, running_toggle.clone(),
        0, 4, 2, 1,
        GridExpand::Both,
        GridAlignment::Fill,
        GridAlignment::Fill
    );

    // イベント処理

    let mut window = Window::new(&ui, "音割れさせるやつ", 400, 160, WindowType::NoMenubar);
    window.set_child(&ui, root_grid);
    window.show(&ui);
    ui.main();
}

pub fn wake_up_main_thread() {
    extern "C" fn do_nothing(_data: *mut libc::c_void) { }

    unsafe {
        // PostMessage するだけなので、初期化されていなくても、どこのスレッドから呼んでも死なない
        ui_sys::uiQueueMain(do_nothing, std::ptr::null_mut());
    }
}
