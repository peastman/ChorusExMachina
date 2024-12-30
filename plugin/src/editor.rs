use crate::{ChorusExMachinaParams, VoicePart};
use chorus::director::Message;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use std::sync::{Arc, Mutex, mpsc};

pub fn draw_editor(params: Arc<ChorusExMachinaParams>, sender: Arc<Mutex<mpsc::Sender<Message>>>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        (),
        |_, _| {},
        move |egui_ctx, setter, _state| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                let mut new_voice_part = params.voice_part.value();
                let mut new_voice_count = params.voice_count.value();
                ui.horizontal(|ui| {
                    ui.label("Voice Part");
                    egui::ComboBox::from_id_source("Voice Part").selected_text(format!("{:?}", new_voice_part)).show_ui(ui, |ui| {
                        ui.selectable_value(&mut new_voice_part, VoicePart::Soprano, "Soprano");
                        ui.selectable_value(&mut new_voice_part, VoicePart::Alto, "Alto");
                        ui.selectable_value(&mut new_voice_part, VoicePart::Tenor, "Tenor");
                        ui.selectable_value(&mut new_voice_part, VoicePart::Bass, "Bass");
                    });
                    ui.add_space(10.0);
                    ui.label("Voices");
                    ui.add(egui::Slider::new(&mut new_voice_count, 1..=8));
                });
                if params.voice_part.value() != new_voice_part || params.voice_count.value() != new_voice_count {
                    setter.begin_set_parameter(&params.voice_part);
                    setter.set_parameter(&params.voice_part, new_voice_part);
                    setter.end_set_parameter(&params.voice_part);
                    setter.begin_set_parameter(&params.voice_count);
                    setter.set_parameter(&params.voice_count, new_voice_count);
                    setter.end_set_parameter(&params.voice_count);
                    let voice_part = match &new_voice_part {
                        VoicePart::Soprano => chorus::VoicePart::Soprano,
                        VoicePart::Alto => chorus::VoicePart::Alto,
                        VoicePart::Tenor => chorus::VoicePart::Tenor,
                        VoicePart::Bass => chorus::VoicePart::Bass,
                    };
                    let _ = sender.lock().unwrap().send(Message::Reinitialize {voice_part: voice_part, voice_count: new_voice_count as usize});
                };
                egui::Grid::new("sliders").show(ui, |ui| {
                    draw_param_slider(ui, &params.dynamics, setter);
                    draw_param_slider(ui, &params.vibrato, setter);
                    draw_param_slider(ui, &params.stereo_width, setter);
                });
            });
        },
    )
}

fn draw_param_slider(ui: &mut egui::Ui, param: &FloatParam, setter: &ParamSetter) -> bool {
    ui.label(param.name());
    let mut value = param.value();
    let mut changed = false;
    if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)).dragged() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, value);
        setter.end_set_parameter(param);
        changed = true;
    }
    ui.end_row();
    changed
}