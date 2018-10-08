#![windows_subsystem = "windows"]

extern crate cpal;
extern crate env_logger;
extern crate iui;
#[macro_use] extern crate log;
extern crate libc;
extern crate rodio;
extern crate ui_sys;

mod effects;
mod player;

#[cfg(windows)]
mod windows_helper;

use std::cell::{Cell, RefCell};
use iui::controls::*;
use iui::prelude::*;
use self::player::OtowarePlayer;

fn main() {
    env_logger::init();

    let ui = UI::init().unwrap();

    // レイアウト
    let mut root_grid = LayoutGrid::new(&ui);

    let (mut input_combo, mut output_combo, mut gain_slider, mut volume_slider) = {
        let mut append_input_control = |row, label, control: Control| {
            root_grid.append(
                &ui, Label::new(&ui, label),
                0, row, 1, 1,
                GridExpand::Neither,
                GridAlignment::Start,
                GridAlignment::Center
            );
            root_grid.append(
                &ui, control,
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

        let gain_slider = Slider::new(&ui, 0, 100);
        append_input_control(2, "ゲイン: ", gain_slider.clone().into());

        let mut volume_slider = Slider::new(&ui, 0, 100);
        volume_slider.set_value(&ui, 50);
        append_input_control(3, "音量: ", volume_slider.clone().into());

        (input_combo, output_combo, gain_slider, volume_slider)
    };

    let mut playing_toggle = Checkbox::new(&ui, "音割れさせる");
    // Windows なら、ボタンの見た目に書き換える
    #[cfg(windows)]
    windows_helper::make_push_like(&mut playing_toggle);
    root_grid.append(
        &ui, playing_toggle.clone(),
        0, 4, 2, 1,
        GridExpand::Both,
        GridAlignment::Fill,
        GridAlignment::Fill
    );

    // 入力デバイス一覧を取得
    let input_devices: Vec<cpal::Device> = cpal::input_devices().collect();
    for device in input_devices.iter() {
        input_combo.append(&ui, &device.name());
    }

    // デフォルト入力デバイス
    // （本当は ID で比較したいんじゃ～～）
    let selected_input_device_index =
        cpal::default_input_device()
            .and_then(|default_device| {
                let default_device_name = default_device.name();
                input_devices.iter().enumerate()
                    .find(|(_, device)| device.name() == default_device_name)
            })
            .map(|(index, _)| index);

    if let Some(index) = selected_input_device_index {
        input_combo.set_selected(&ui, index as i64);
    }

    let selected_input_device_index = Cell::new(selected_input_device_index);

    // 出力デバイス一覧を取得
    let output_devices: Vec<rodio::Device> = rodio::output_devices().collect();
    for device in output_devices.iter() {
        output_combo.append(&ui, &device.name());
    }

    // デフォルト出力デバイス
    let selected_output_device_index =
        rodio::default_output_device()
            .and_then(|default_device| {
                let default_device_name = default_device.name();
                output_devices.iter().enumerate()
                    .find(|(_, device)| device.name() == default_device_name)
            })
            .map(|(index, _)| index);

    if let Some(index) = selected_output_device_index {
        output_combo.set_selected(&ui, index as i64);
    }

    let selected_output_device_index = Cell::new(selected_output_device_index);

    // プレイヤー状態管理
    let player = RefCell::new(OtowarePlayer::new());
    let mut playing = false;

    let mut ui_clone = ui.clone();
    let mut playing_toggle_clone = playing_toggle.clone();
    let mut update_state = {
        |mut update_input, mut update_output| {
            let mut player = player.borrow_mut();
            match (selected_input_device_index.get(), selected_output_device_index.get()) {
                (Some(input_device_index), Some(output_device_index)) => {
                    let new_playing = playing_toggle_clone.checked(&ui_clone);
                    match (playing, new_playing) {
                        (false, true) => {
                            // 新たに再生開始
                            update_input = true;
                            update_output = true;
                        }
                        (true, true) => {
                            // 引数の update_input, update_output を使用
                        }
                        (_, false) => {
                            // 停止
                            player.clear();
                            update_input = false;
                            update_output = false;
                        }
                    };

                    ui_clone.set_enabled(playing_toggle_clone.clone(), true);
                    playing = new_playing;

                    // 変更を player に反映
                    if update_input {
                        if let Err(err) = player.set_input(&input_devices[input_device_index]) {
                            error!("{:?}", err);

                            // 失敗したので、停止状態にする
                            update_output = false;
                            playing = false;
                            player.clear();
                            playing_toggle_clone.set_checked(&ui_clone, false);
                        }
                    }

                    if update_output {
                        player.set_output(&output_devices[output_device_index]);
                    }
                }
                _ => {
                    // 再生不可能なので、すべて停止
                    playing = false;
                    player.clear();
                    playing_toggle_clone.set_checked(&ui_clone, false);
                    ui_clone.set_enabled(playing_toggle_clone.clone(), false);
                    return;
                }
            }
        }
    };

    update_state(false, false);

    // イベント処理
    input_combo.on_selected(&ui, |index| {
        let new_value =
            match index as usize {
                x if x < input_devices.len() => Some(x),
                _ => None,
            };
        if selected_input_device_index.replace(new_value) != new_value {
            update_state(true, false);
        }
    });

    output_combo.on_selected(&ui, |index| {
        let new_value =
            match index as usize {
                x if x < output_devices.len() => Some(x),
                _ => None,
            };
        if selected_output_device_index.replace(new_value) != new_value {
            update_state(false, true);
        }
    });

    gain_slider.on_changed(&ui, |value| {
        assert!(value >= 0 && value <= 100);
        player.borrow_mut().set_gain(value as u8);
    });

    volume_slider.on_changed(&ui, |value| {
        assert!(value >= 0 && value <= 100);
        player.borrow_mut().set_volume(value as u8);
    });

    playing_toggle.on_toggled(&ui, |_| update_state(false, false));

    let mut window = Window::new(&ui, "音割れさせるやつ", 400, 160, WindowType::NoMenubar);
    window.set_child(&ui, root_grid);
    window.show(&ui);
    ui.main();
}
