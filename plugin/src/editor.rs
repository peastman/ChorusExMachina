use crate::{ChorusExMachinaParams, VoicePart};
use chorus::director::Message;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use egui_extras::{Column, TableBuilder};
use std::sync::{Arc, Mutex, mpsc};

#[derive(PartialEq)]
enum Panel {
    Controls,
    Words,
    Help,
    About
}

pub struct UIState {
    current_panel: Panel
}

impl UIState {
    pub fn new() -> Self {
        Self {
            current_panel: Panel::Controls
        }
    }
}

pub fn draw_editor(params: Arc<ChorusExMachinaParams>, sender: Arc<Mutex<mpsc::Sender<Message>>>, state: Arc<Mutex<UIState>>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        (),
        |_, _| {},
        move |ctx, setter, _state| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::SidePanel::left("tabs").show_inside(ui, |ui| {
                    let mut state = state.lock().unwrap();
                    ui.selectable_value(&mut state.current_panel, Panel::Controls, "Controls");
                    ui.selectable_value(&mut state.current_panel, Panel::Words, "Words");
                    ui.selectable_value(&mut state.current_panel, Panel::Help, "Help");
                    ui.selectable_value(&mut state.current_panel, Panel::About, "About");
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let state = state.lock().unwrap();
                    match state.current_panel {
                        Panel::Controls => draw_controls_panel(ui, &params, &sender, setter),
                        Panel::Words => draw_words_panel(ui, &params, &sender, setter),
                        Panel::Help => draw_help_panel(ui),
                        Panel::About => draw_about_panel(ui)
                    }
                });
            });
        },
    )
}

fn draw_controls_panel(ui: &mut egui::Ui, params: &Arc<ChorusExMachinaParams>, sender: &Arc<Mutex<mpsc::Sender<Message>>>, setter: &ParamSetter) {
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
        draw_param_slider(ui, &params.intensity, setter);
        draw_param_slider(ui, &params.stereo_width, setter);
    });
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

fn draw_words_panel(ui: &mut egui::Ui, params: &Arc<ChorusExMachinaParams>, sender: &Arc<Mutex<mpsc::Sender<Message>>>, setter: &ParamSetter) {
    let table = TableBuilder::new(ui)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .column(Column::remainder())
        .sense(egui::Sense::click());
    table.header(20.0, |mut header| {
        header.col(|ui| {
            ui.strong("Index");
        });
        header.col(|ui| {
            ui.strong("Phrase");
        });
    }).body(|body| {
        body.rows(18.0, 128, |mut row| {
            let mut phrases = params.phrases.lock().unwrap();
            let row_index = row.index();
            let selected_phrase = params.selected_phrase.value() as usize;
            if row_index == selected_phrase {
                row.set_selected(true);
            }
            row.col(|ui| {
                ui.label(format!("{row_index}"));
            });
            row.col(|ui| {
                if row_index == selected_phrase {
                    let response = ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut phrases[row_index]));
                }
                else {
                    ui.label(&phrases[row_index]);
                }
            });
            if row.response().clicked() {
                setter.begin_set_parameter(&params.selected_phrase);
                setter.set_parameter(&params.selected_phrase, row_index as i32);
                setter.end_set_parameter(&params.selected_phrase);
            }
        });
    });
}

fn draw_help_panel(ui: &mut egui::Ui) {
}

fn draw_about_panel(ui: &mut egui::Ui) {
}
