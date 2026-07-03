use krender::{Renderer, config_buffer::{self, ViewMode}};

use crate::ui;

pub struct UiState {
    pub view_mode: ViewMode,
    pub exposure: f32,
    pub max_newton_steps: u32,
    pub max_bounces: u32,

    pub camera_sensitivity: f32,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Normal,
            exposure: 1.0,
            max_newton_steps: 5,
            max_bounces: 8,

            camera_sensitivity: 0.5,
        }
    }

    pub fn render_config(&mut self, ui: &mut egui::Ui, renderer: &mut krender::Renderer) {
        ui.heading(format!("FPS: {:.2}", renderer.fps()));

        ui.heading("Ray casting");
        egui::ComboBox::from_label("Visualization Mode")
            .selected_text(format!("{:?}", self.view_mode))
            .show_ui(ui, |ui| {
                for mode in ViewMode::iter() {
                    ui.selectable_value(&mut self.view_mode, mode, format!("{:?}", mode));
                }
            });

        ui.add(egui::Slider::new(&mut self.exposure, 0.0..=1.0).text("Exposure"));
        ui.add(egui::Slider::new(&mut self.max_newton_steps, 1..=20).text("Max Newton Steps"));
        ui.add(egui::Slider::new(&mut self.max_bounces, 1..=20).text("Max Bounces"));

        ui.heading("Camera");
        let camera = renderer.camera_mut();
        ui.label(format!("Position: {:?}", camera.position()));
        ui.label(format!("Rotation: {:?}", camera.rotation()));
        ui.add(egui::Slider::new(&mut self.camera_sensitivity, 0.0..=5.0).text("Sensitivity"));
        ui.add(egui::Slider::new(&mut camera.speed, 0.1..=20.0).text("Speed"));
    }

    pub fn update_renderer(&self, renderer: &mut Renderer) {
        let config_buffer = renderer.config_buffer_mut();

        config_buffer.set_view_mode(self.view_mode);
        config_buffer.set_exposure(self.exposure);
        config_buffer.set_max_newton_steps(self.max_newton_steps);
        config_buffer.set_max_bounces(self.max_bounces);
    }

    pub fn update_app(&self, camera_sensitivity: &mut f32, camera_speed: &mut f32) {
        *camera_sensitivity = self.camera_sensitivity;
    }
}
