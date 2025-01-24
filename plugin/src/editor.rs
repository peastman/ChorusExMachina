// Copyright 2025 by Peter Eastman
//
// This file is part of Chorus Ex Machina.
//
// Chorus Ex Machina is free software: you can redistribute it and/or modify it under the terms
// of the GNU Lesser General Public License as published by the Free Software Foundation, either
// version 2.1 of the License, or (at your option) any later version.
//
// Chorus Ex Machina is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License along with Chorus Ex Machina.
// If not, see <https://www.gnu.org/licenses/>.

use crate::{ChorusExMachinaParams, VoicePart};
use chorus::director::Message;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui};
use egui_extras::{Column, TableBuilder};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::{Arc, Mutex, mpsc};

#[derive(PartialEq)]
enum Panel {
    Controls,
    Text,
    Help,
    About
}

pub struct UIState {
    current_panel: Panel,
    edit_phrase: usize
}

impl UIState {
    pub fn new() -> Self {
        Self {
            current_panel: Panel::Controls,
            edit_phrase: 0
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
                egui::SidePanel::left("tabs").max_width(100.0).resizable(false).show_inside(ui, |ui| {
                    let mut state = state.lock().unwrap();
                    ui.vertical_centered_justified(|ui| {
                        ui.selectable_value(&mut state.current_panel, Panel::Controls, "Controls");
                        ui.selectable_value(&mut state.current_panel, Panel::Text, "Text");
                        ui.selectable_value(&mut state.current_panel, Panel::Help, "Help");
                        ui.selectable_value(&mut state.current_panel, Panel::About, "About");
                    })
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let mut state = state.lock().unwrap();
                    match state.current_panel {
                        Panel::Controls => draw_controls_panel(ui, &params, &sender, setter),
                        Panel::Text => draw_text_panel(ui, &params, setter, &mut state),
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
    ui.label(egui::RichText::new("The voices in the chorus").italics());
    ui.add_space(5.0);
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
    ui.add_space(20.0);
    ui.label(egui::RichText::new("These controls can be mapped to MIDI CCs and automated in a DAW").italics());
    ui.add_space(5.0);
    egui::Grid::new("sliders").show(ui, |ui| {
        draw_param_slider(ui, &params.dynamics, setter);
        draw_param_slider(ui, &params.vibrato, setter);
        draw_param_slider(ui, &params.intensity, setter);
        draw_param_slider(ui, &params.brightness, setter);
        draw_param_slider(ui, &params.consonant_volume, setter);
        draw_param_slider(ui, &params.attack_rate, setter);
        draw_param_slider(ui, &params.stereo_width, setter);
        ui.label("Vowel Delay (ms)");
        let mut delay = params.vowel_delay.value();
        if ui.add(egui::Slider::new(&mut delay, 0..=200)).changed() {
            setter.begin_set_parameter(&params.vowel_delay);
            setter.set_parameter(&params.vowel_delay, delay);
            setter.end_set_parameter(&params.vowel_delay);
        }
        ui.end_row();
        let mut accent = params.accent.value();
        if ui.checkbox(&mut accent, "Accent").changed() {
            setter.begin_set_parameter(&params.accent);
            setter.set_parameter(&params.accent, accent);
            setter.end_set_parameter(&params.accent);
        }
        let mut advance_syllable = params.advance_syllable.value();
        if ui.checkbox(&mut advance_syllable, "Advance Syllable").changed() {
            setter.begin_set_parameter(&params.advance_syllable);
            setter.set_parameter(&params.advance_syllable, advance_syllable);
            setter.end_set_parameter(&params.advance_syllable);
        }
    });
}

fn draw_param_slider(ui: &mut egui::Ui, param: &FloatParam, setter: &ParamSetter) {
    ui.label(param.name());
    let mut value = param.value();
    if ui.add(egui::Slider::new(&mut value, 0.0..=1.0)).changed() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, value);
        setter.end_set_parameter(param);
    }
    ui.end_row();
}

fn draw_text_panel(ui: &mut egui::Ui, params: &Arc<ChorusExMachinaParams>, setter: &ParamSetter, state: &mut UIState) {
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
            let mut clicked = false;
            if row_index == selected_phrase {
                row.set_selected(true);
            }
            row.col(|ui| {
                ui.label(format!("{row_index}"));
            });
            row.col(|ui| {
                if row_index == state.edit_phrase {
                    clicked = ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut phrases[row_index])).clicked();
                }
                else {
                    clicked = ui.label(&phrases[row_index]).clicked();
                }
            });
            clicked |= row.response().clicked();
            if clicked {
                setter.begin_set_parameter(&params.selected_phrase);
                setter.set_parameter(&params.selected_phrase, row_index as i32);
                setter.end_set_parameter(&params.selected_phrase);
                state.edit_phrase = row_index;
            }
        });
    });
}

fn draw_help_panel(ui: &mut egui::Ui) {
    let mut cache = CommonMarkCache::default();
    let text = include_str!("help.md");
    egui::ScrollArea::vertical().show(ui, |ui| {
        CommonMarkViewer::new("help").show(ui, &mut cache, text);
    });
}

fn draw_about_panel(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(30.0);
        ui.label(egui::RichText::new("Chorus Ex Machina").size(36.0).italics());
        ui.label(egui::RichText::new(format!("version {}", env!("CARGO_PKG_VERSION"))).size(14.0));
        ui.label(egui::RichText::new("Copyright 2025 by Peter Eastman").size(14.0));
        ui.add_space(12.0);
        ui.hyperlink("https://github.com/peastman/ChorusExMachina");
    });
}
