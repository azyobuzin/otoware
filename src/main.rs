#![allow(dead_code)]

extern crate cpal;
extern crate iui;
extern crate rodio;

mod effects;
mod windows_helper;

use iui::controls;
use iui::prelude::*;

fn main() {
    let ui = UI::init().unwrap();

    let mut root_vbox = controls::VerticalBox::new(&ui);

    let mut input_combo = controls::Combobox::new(&ui);
    root_vbox.append(
        &ui,
        create_labeled_box(&ui, input_combo.clone(), "入力: "),
        LayoutStrategy::Compact
    );

    let mut output_combo = controls::Combobox::new(&ui);
    root_vbox.append(
        &ui,
        create_labeled_box(&ui, output_combo.clone(), "出力: "),
        LayoutStrategy::Compact
    );

    let mut gain_slider = controls::Slider::new(&ui, 0, 80);
    root_vbox.append(
        &ui,
        create_labeled_box(&ui, gain_slider.clone(), "ゲイン: "),
        LayoutStrategy::Compact
    );

    root_vbox.append(&ui, controls::Spacer::new(&ui), LayoutStrategy::Stretchy);

    let mut running_toggle = controls::Checkbox::new(&ui, "音割れさせる");
    windows_helper::make_push_like(&mut running_toggle);
    root_vbox.append(&ui, running_toggle.clone(), LayoutStrategy::Compact);

    let mut window = Window::new(&ui, "音割れさせるやつ", 400, 140, WindowType::NoMenubar);
    window.set_child(&ui, root_vbox);
    window.show(&ui);
    ui.main();
}

fn create_labeled_box(ui: &UI, control: impl Into<controls::Control>, label: &str) -> controls::HorizontalBox
{
    let label_control = controls::Label::new(ui, label);
    let mut hbox = controls::HorizontalBox::new(ui);
    hbox.append(ui, label_control, LayoutStrategy::Compact);
    hbox.append(ui, control, LayoutStrategy::Stretchy);
    hbox
}
