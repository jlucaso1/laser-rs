use eframe::egui;

use super::canvas::{CanvasState, Tool, render_canvas};
use super::history::History;
use super::svg_doc::SvgDocument;

pub struct SvgEditorApp {
    document: SvgDocument,
    canvas_state: CanvasState,
    history: History,
    status_message: String,
    /// Track if we're currently dragging (to save state only once per drag)
    drag_state_saved: bool,
}

impl Default for SvgEditorApp {
    fn default() -> Self {
        Self {
            document: SvgDocument::new(),
            canvas_state: CanvasState::new(),
            history: History::new(),
            status_message: String::from("Ready - Open an SVG file to begin editing"),
            drag_state_saved: false,
        }
    }
}

impl SvgEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SVG files", &["svg"])
            .pick_file()
        {
            match SvgDocument::load(&path) {
                Ok(doc) => {
                    let elem_count = doc.elements.len();
                    self.document = doc;
                    self.canvas_state = CanvasState::new();
                    self.history.clear();
                    // Center the document
                    self.canvas_state.pan = egui::Vec2::new(50.0, 50.0);
                    self.status_message =
                        format!("Loaded: {} ({} elements)", path.display(), elem_count);
                }
                Err(e) => {
                    self.status_message = format!("Error loading file: {}", e);
                }
            }
        }
    }

    fn undo(&mut self) {
        if let Some(doc) = self.history.undo(&self.document) {
            self.document = doc;
            self.canvas_state.selected_element = None;
            self.canvas_state.selected_point = None;
            self.status_message = format!("Undo ({} more available)", self.history.undo_count());
        }
    }

    fn redo(&mut self) {
        if let Some(doc) = self.history.redo(&self.document) {
            self.document = doc;
            self.canvas_state.selected_element = None;
            self.canvas_state.selected_point = None;
            self.status_message = format!("Redo ({} more available)", self.history.redo_count());
        }
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open SVG").clicked() {
                self.open_file();
            }

            ui.separator();

            // Undo/Redo buttons
            if ui
                .add_enabled(self.history.can_undo(), egui::Button::new("↶ Undo"))
                .on_hover_text("Ctrl+Z")
                .clicked()
            {
                self.undo();
            }
            if ui
                .add_enabled(self.history.can_redo(), egui::Button::new("↷ Redo"))
                .on_hover_text("Ctrl+Y or Ctrl+Shift+Z")
                .clicked()
            {
                self.redo();
            }

            ui.separator();

            ui.label("Tool:");
            if ui
                .selectable_label(self.canvas_state.current_tool == Tool::Select, "Select")
                .clicked()
            {
                self.canvas_state.current_tool = Tool::Select;
            }
            if ui
                .selectable_label(self.canvas_state.current_tool == Tool::Move, "Move")
                .clicked()
            {
                self.canvas_state.current_tool = Tool::Move;
            }

            ui.separator();

            ui.label(format!("Zoom: {:.0}%", self.canvas_state.zoom * 100.0));

            if ui.button("Reset View").clicked() {
                self.canvas_state.pan = egui::Vec2::new(50.0, 50.0);
                self.canvas_state.zoom = 1.0;
            }

            ui.separator();

            if let Some(idx) = self.canvas_state.selected_element {
                if let Some(elem) = self.document.elements.get(idx) {
                    ui.label(format!("Selected: {} ({})", elem.id(), idx));
                }
            } else {
                ui.label("No selection");
            }
        });
    }

    fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Elements");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, element) in self.document.elements.iter().enumerate() {
                let is_selected = self.canvas_state.selected_element == Some(idx);
                let label = format!("{}: {}", idx, element.id());

                if ui.selectable_label(is_selected, label).clicked() {
                    self.canvas_state.selected_element = Some(idx);
                    self.canvas_state.selected_point = None;
                }
            }
        });

        ui.separator();
        ui.heading("Properties");

        if let Some(idx) = self.canvas_state.selected_element
            && let Some(element) = self.document.elements.get(idx)
        {
            let (min, max) = element.bounds();
            ui.label(format!(
                "Bounds: ({:.1}, {:.1}) - ({:.1}, {:.1})",
                min.x, min.y, max.x, max.y
            ));
            let center = element.center();
            ui.label(format!("Center: ({:.1}, {:.1})", center.x, center.y));
        }

        ui.separator();
        ui.heading("History");
        ui.label(format!(
            "Undo: {} | Redo: {}",
            self.history.undo_count(),
            self.history.redo_count()
        ));
    }
}

impl eframe::App for SvgEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        let mut do_undo = false;
        let mut do_redo = false;

        ctx.input(|i| {
            if i.key_pressed(egui::Key::O) && i.modifiers.command {
                // Open file is handled separately due to borrow issues
            }
            if i.key_pressed(egui::Key::Escape) {
                self.canvas_state.selected_element = None;
                self.canvas_state.selected_point = None;
            }
            // Undo: Ctrl+Z
            if i.key_pressed(egui::Key::Z) && i.modifiers.command && !i.modifiers.shift {
                do_undo = true;
            }
            // Redo: Ctrl+Y or Ctrl+Shift+Z
            if i.key_pressed(egui::Key::Y) && i.modifiers.command {
                do_redo = true;
            }
            if i.key_pressed(egui::Key::Z) && i.modifiers.command && i.modifiers.shift {
                do_redo = true;
            }
        });

        if do_undo {
            self.undo();
        }
        if do_redo {
            self.redo();
        }

        // Track drag state to save history at the right time
        let was_dragging = self.canvas_state.dragging;

        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.render_toolbar(ui);
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "Document: {:.0} x {:.0}",
                        self.document.width, self.document.height
                    ));
                });
            });
        });

        // Side panel for element list
        egui::SidePanel::left("elements_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.render_side_panel(ui);
            });

        // Main canvas area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Save state when drag starts
            if self.canvas_state.dragging && !self.drag_state_saved {
                self.history.save_state(&self.document);
                self.drag_state_saved = true;
            }

            render_canvas(ui, &mut self.document, &mut self.canvas_state);

            // Reset drag state saved flag when drag ends
            if was_dragging && !self.canvas_state.dragging {
                self.drag_state_saved = false;
            }
        });

        // Request continuous repaint for smooth interaction
        ctx.request_repaint();
    }
}
