//! Style controls of the editor (no selection yet): the current style with
//! switchable overrides for new geometry and the brush, and the Styles
//! window that edits the drawing's style table through commands.
//!
//! The current style is UI state, not part of the drawing, and is not saved.

use constraint_runtime::{
    Arrow, Dash, Rgb, Style, StylePatch, StyleProps, StyleRef, Styles, DEFAULT_STYLE,
};
use eframe::egui;
use sketch_session::Command;

const DASHES: [(Dash, &str); 4] = [
    (Dash::Solid, "solid"),
    (Dash::Dashed, "dashed"),
    (Dash::Dotted, "dotted"),
    (Dash::DashDot, "dashdot"),
];

const ARROWS: [(Arrow, &str); 4] = [
    (Arrow::None, "none"),
    (Arrow::Start, "start"),
    (Arrow::End, "end"),
    (Arrow::Both, "both"),
];

/// One override of the current look: switched on or off, with the value it
/// has when on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Switch<T> {
    pub on: bool,
    pub value: T,
}

impl<T: Copy> Switch<T> {
    fn off(value: T) -> Self {
        Self { on: false, value }
    }

    fn get(self) -> Option<T> {
        self.on.then_some(self.value)
    }
}

/// The style and overrides new geometry gets and the brush applies.
#[derive(Clone, Debug, PartialEq)]
pub struct CurrentStyle {
    pub style: String,
    pub width: Switch<f64>,
    pub color: Switch<Rgb>,
    pub dash: Switch<Dash>,
    pub arrow: Switch<Arrow>,
}

impl Default for CurrentStyle {
    fn default() -> Self {
        let b = Style::BUILT_IN;
        Self {
            style: DEFAULT_STYLE.to_owned(),
            width: Switch::off(b.width),
            color: Switch::off(b.color),
            dash: Switch::off(b.dash),
            arrow: Switch::off(b.arrow),
        }
    }
}

impl CurrentStyle {
    /// Style reference and switched-on overrides for a new geometry.
    pub fn look(&self) -> StyleRef {
        StyleRef {
            style: self.style.clone(),
            overrides: StyleProps {
                width: self.width.get(),
                color: self.color.get(),
                dash: self.dash.get(),
                arrow: self.arrow.get(),
            },
        }
    }

    /// `set_style` that makes geometry `name` look like a new geometry drawn
    /// now: the current style, switched-on overrides set, the others removed.
    pub fn brush(&self, name: &str) -> Command {
        let o = self.look().overrides;
        Command::SetStyle {
            name: name.to_owned(),
            style: Some(self.style.clone()),
            patch: StylePatch {
                width: Some(o.width),
                color: Some(o.color),
                dash: Some(o.dash),
                arrow: Some(o.arrow),
            },
        }
    }

    /// Fall back to `default` when the current style is gone from `styles`.
    pub fn keep_valid(&mut self, styles: &Styles) {
        if !styles.contains(&self.style) {
            self.style = DEFAULT_STYLE.to_owned();
        }
    }

    /// The toolbar's style row.
    pub fn row(&mut self, ui: &mut egui::Ui, styles: &Styles) {
        ui.label("Style:");
        egui::ComboBox::from_id_salt("current_style")
            .selected_text(self.style.as_str())
            .show_ui(ui, |ui| {
                for (name, _) in styles.iter() {
                    ui.selectable_value(&mut self.style, name.to_owned(), name);
                }
            });
        ui.checkbox(&mut self.width.on, "width");
        ui.add_enabled(self.width.on, width_drag(&mut self.width.value));
        ui.checkbox(&mut self.color.on, "colour");
        ui.add_enabled_ui(self.color.on, |ui| color_button(ui, &mut self.color.value));
        ui.checkbox(&mut self.dash.on, "dash");
        ui.add_enabled_ui(self.dash.on, |ui| {
            choice(ui, "current_dash", &mut self.dash.value, &DASHES)
        });
        ui.checkbox(&mut self.arrow.on, "arrow");
        ui.add_enabled_ui(self.arrow.on, |ui| {
            choice(ui, "current_arrow", &mut self.arrow.value, &ARROWS)
        });
    }
}

fn width_drag(width: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(width)
        .speed(0.02)
        .range(0.05..=50.0)
        .max_decimals(2)
        .suffix(" pt")
        .update_while_editing(false)
}

fn color_button(ui: &mut egui::Ui, color: &mut Rgb) -> egui::Response {
    let mut c = [color.0, color.1, color.2];
    let r = egui::color_picker::color_edit_button_srgb(ui, &mut c);
    *color = Rgb(c[0], c[1], c[2]);
    r
}

fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash,
    value: &mut T,
    options: &[(T, &str)],
) {
    let text = options
        .iter()
        .find(|(v, _)| v == value)
        .map_or("?", |(_, t)| *t);
    egui::ComboBox::from_id_salt(id)
        .selected_text(text)
        .show_ui(ui, |ui| {
            for &(v, t) in options {
                ui.selectable_value(value, v, t);
            }
        });
}

/// State of the "Styles…" window.
#[derive(Default)]
pub struct StylesWindow {
    pub open: bool,
    /// Name typed for a new style.
    new_name: String,
    /// Edit of one style shown but not yet committed (while dragging or
    /// while a colour picker is open).
    pending: Option<(String, StylePatch)>,
}

impl StylesWindow {
    /// Show the window over `styles`; returns the command of a finished edit.
    pub fn show(&mut self, ctx: &egui::Context, styles: &Styles) -> Option<Command> {
        let mut command = None;
        let mut open = self.open;
        egui::Window::new("Styles")
            .open(&mut open)
            .anchor(egui::Align2::LEFT_TOP, [16.0, 110.0])
            .show(ctx, |ui| {
                egui::Grid::new("styles").striped(true).show(ui, |ui| {
                    for h in ["style", "width", "colour", "dash", "arrow", ""] {
                        ui.strong(h);
                    }
                    ui.end_row();
                    for (name, props) in styles.iter() {
                        if let Some(c) = self.style_row(ui, name, props) {
                            command = Some(c);
                        }
                        ui.end_row();
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_name)
                            .hint_text("new style")
                            .desired_width(100.0),
                    );
                    if ui.button("Add").clicked() {
                        // A style holds at least one property: start from the built-in width.
                        command = Some(Command::DefineStyle {
                            style: self.new_name.trim().to_owned(),
                            patch: StylePatch {
                                width: Some(Some(Style::BUILT_IN.width)),
                                ..StylePatch::default()
                            },
                        });
                        self.new_name.clear();
                    }
                });
            });
        self.open = open;
        // Commit on release: no button held and no colour picker open.
        let released = !ctx.input(|i| i.pointer.any_down()) && !ctx.memory(|m| m.any_popup_open());
        if command.is_none() && released {
            if let Some((style, patch)) = self.pending.take() {
                command = Some(Command::DefineStyle { style, patch });
            }
        }
        command
    }

    /// One row of the table; edits go to `pending`, delete returns a command.
    fn style_row(&mut self, ui: &mut egui::Ui, name: &str, props: &StyleProps) -> Option<Command> {
        let mut patch = match &self.pending {
            Some((n, p)) if n == name => p.clone(),
            _ => StylePatch::default(),
        };
        let shown = patch.applied_to(props);
        let b = Style::BUILT_IN;
        ui.label(name);
        ui.horizontal(|ui| {
            let mut on = shown.width.is_some();
            let mut w = shown.width.unwrap_or(b.width);
            let toggled = ui.checkbox(&mut on, "").changed();
            let dragged = ui.add_enabled(on, width_drag(&mut w)).changed();
            if toggled || dragged {
                patch.width = Some(on.then_some(w));
            }
        });
        ui.horizontal(|ui| {
            let mut on = shown.color.is_some();
            let mut c = shown.color.unwrap_or(b.color);
            let toggled = ui.checkbox(&mut on, "").changed();
            let picked = ui
                .add_enabled_ui(on, |ui| color_button(ui, &mut c).changed())
                .inner;
            if toggled || picked {
                patch.color = Some(on.then_some(c));
            }
        });
        ui.horizontal(|ui| {
            let mut on = shown.dash.is_some();
            let mut d = shown.dash.unwrap_or(b.dash);
            let before = d;
            let toggled = ui.checkbox(&mut on, "").changed();
            ui.add_enabled_ui(on, |ui| choice(ui, ("dash", name), &mut d, &DASHES));
            if toggled || d != before {
                patch.dash = Some(on.then_some(d));
            }
        });
        ui.horizontal(|ui| {
            let mut on = shown.arrow.is_some();
            let mut a = shown.arrow.unwrap_or(b.arrow);
            let before = a;
            let toggled = ui.checkbox(&mut on, "").changed();
            ui.add_enabled_ui(on, |ui| choice(ui, ("arrow", name), &mut a, &ARROWS));
            if toggled || a != before {
                patch.arrow = Some(on.then_some(a));
            }
        });
        if !patch.is_empty() {
            self.pending = Some((name.to_owned(), patch));
        }
        let delete = ui.add_enabled(name != DEFAULT_STYLE, egui::Button::new("Delete"));
        delete.clicked().then(|| Command::DeleteStyle {
            style: name.to_owned(),
        })
    }
}

#[cfg(test)]
#[path = "tests_style_ui.rs"]
mod tests_style_ui;
