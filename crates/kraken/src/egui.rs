use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use egui::Context as EguiContext;
use egui_glow::Painter;
use egui_winit::State;
use gtw::Gpu;
use krender::config_buffer::ViewMode;
use winit::event_loop::ActiveEventLoop;
use winit::window;

use crate::viewport::Viewport;

pub struct ConfigPanel {
    viewport: Rc<RefCell<Viewport>>,

    view_mode: ViewMode,
    exposure: f32,
    max_newton_steps: u32,
    max_bounces: u32,

    camera_sensitivity: f32,
    camera_speed: f32,
}

impl ConfigPanel {
    pub fn new(viewport: Rc<RefCell<Viewport>>) -> Self {
        let viewport_copy = Rc::clone(&viewport);
        let viewport_lock = viewport.borrow();
        let config_buffer = viewport_lock.config_buffer();

        Self {
            viewport: viewport_copy,

            view_mode: config_buffer.view_mode(),
            exposure: config_buffer.exposure(),
            max_newton_steps: config_buffer.max_newton_steps(),
            max_bounces: config_buffer.max_bounces(),

            camera_sensitivity: viewport_lock.camera().sensitivity(),
            camera_speed: viewport_lock.camera().speed(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        let mut viewport = self.viewport.borrow_mut();

        ui.heading(format!("FPS: {:.2}", viewport.fps()));

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
        let camera = viewport.camera_mut();
        ui.label(format!("Position: {:?}", camera.transform().position));
        ui.label(format!("Rotation: {:?}", camera.transform().rotation));
        ui.add(egui::Slider::new(&mut self.camera_sensitivity, 0.0..=5.0).text("Sensitivity"));
        ui.add(egui::Slider::new(&mut self.camera_speed, 0.1..=20.0).text("Speed"));
    }

    pub fn update(&self) {
        let mut viewport = self.viewport.borrow_mut();
        let config_buffer = viewport.config_buffer_mut();

        config_buffer.set_view_mode(self.view_mode);
        config_buffer.set_exposure(self.exposure);
        config_buffer.set_max_newton_steps(self.max_newton_steps);
        config_buffer.set_max_bounces(self.max_bounces);

        *viewport.camera_mut().sensitivity_mut() = self.camera_sensitivity;
        *viewport.camera_mut().speed_mut() = self.camera_speed;
    }
}

pub struct EguiState {
    painter: Painter,
    egui_state: State,
    egui_ctx: EguiContext,

    config_panel: ConfigPanel,
}

impl EguiState {
    pub fn new(
        gpu: Arc<Gpu>,
        event_loop: &ActiveEventLoop,
        viewport: Rc<RefCell<Viewport>>,
    ) -> Self {
        let egui_ctx = EguiContext::default();

        let painter = egui_glow::Painter::new(gpu.context(), "", None, false)
            .expect("Failed to create egui painter");

        let state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            None,
            None,
            None,
        );

        Self {
            egui_ctx,
            painter,
            egui_state: state,
            config_panel: ConfigPanel::new(viewport),
        }
    }

    pub fn consume_window_event(
        &mut self,
        window: &window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        let response = self.egui_state.on_window_event(window, event);
        response.consumed
    }

    pub fn update(&mut self, window: &window::Window) {
        let raw_input = self.egui_state.take_egui_input(window);

        let full_output = self.egui_ctx.run_ui(raw_input, |ctx| {
            egui::Window::new("Config").show(ctx, |ui| {
                self.config_panel.render(ui);
            });
        });

        self.config_panel.update();

        self.egui_state
            .handle_platform_output(window, full_output.platform_output);

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        self.painter.paint_and_update_textures(
            [window.inner_size().width, window.inner_size().height],
            full_output.pixels_per_point,
            &clipped_primitives,
            &full_output.textures_delta,
        );
    }
}
