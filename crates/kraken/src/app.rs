use egui::Context as EguiContext;
use egui_glow::Painter;
use egui_winit::State;
use glow::{Context, HasContext};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::PossiblyCurrentContext;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SurfaceAttributesBuilder;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::DisplayBuilder;
use kmath::{Vec2f, Vec3f};
use krender::Renderer;
use raw_window_handle::HasWindowHandle;
use std::collections::HashSet;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;
use winit::window::WindowAttributes;

use crate::ui;

pub struct App {
    pub renderer: Option<Renderer>,

    ui_state: ui::UiState,
    painter: Option<Painter>,
    egui_state: Option<State>,
    egui_ctx: EguiContext,

    gl: Option<Arc<Context>>,
    context: Option<PossiblyCurrentContext>,
    surface: Option<Surface<WindowSurface>>,
    window: Option<Window>,

    pressed_keys: HashSet<KeyCode>,
    pub camera_speed: f32,
    pub camera_sensitivity: f32,
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,

            ui_state: ui::UiState::new(),
            window: None,
            gl: None,
            context: None,
            surface: None,

            egui_ctx: EguiContext::default(),
            egui_state: None,
            painter: None,

            pressed_keys: HashSet::new(),
            camera_speed: 5.0,
            camera_sensitivity: 0.5,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("Kraken")
            .with_inner_size(PhysicalSize::new(800, 600));

        let template = ConfigTemplateBuilder::new();
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let window = window.unwrap();

        let raw_window_handle = window.window_handle().unwrap();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(None))
            .build(Some(raw_window_handle.as_raw()));

        let display = gl_config.display();

        let not_current = unsafe {
            display
                .create_context(&gl_config, &context_attributes)
                .unwrap()
        };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle.as_raw(),
            NonZeroU32::new(window.inner_size().width.max(1)).unwrap(),
            NonZeroU32::new(window.inner_size().height.max(1)).unwrap(),
        );

        let surface = unsafe { display.create_window_surface(&gl_config, &attrs).unwrap() };

        let context = not_current.make_current(&surface).unwrap();

        gl::load_with(|s| display.get_proc_address(&CString::new(s).unwrap()).cast());

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                display.get_proc_address(&CString::new(s).unwrap())
            })
        };

        let gl = Arc::new(gl);

        let painter = egui_glow::Painter::new(gl.clone(), "", None, false)
            .expect("Failed to create egui painter");

        let state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            None,
            None,
            None,
        );

        self.window = Some(window);
        self.surface = Some(surface);
        self.context = Some(context);
        self.gl = Some(gl);
        self.painter = Some(painter);
        self.egui_state = Some(state);
        self.renderer = Some(
            Renderer::new(
                self.gl.as_ref().unwrap().clone(),
                Vec2f::new(
                    self.window.as_ref().unwrap().inner_size().width as f32,
                    self.window.as_ref().unwrap().inner_size().height as f32,
                ),
            )
            .expect("Failed creating renderer"),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if Some(window_id) != self.window.as_ref().map(Window::id) {
            return;
        }

        if let Some(state) = &mut self.egui_state {
            let response = state.on_window_event(self.window.as_ref().unwrap(), &event);
            if response.consumed {
                return;
            }
        }

        match event {
            WindowEvent::Resized(size) => {
                self.surface.as_ref().unwrap().resize(
                    self.context.as_ref().unwrap(),
                    NonZeroU32::new(size.width.max(1)).unwrap(),
                    NonZeroU32::new(size.height.max(1)).unwrap(),
                );

                let scale = self.window.as_ref().unwrap().scale_factor();

                let width = (size.width as f64 * scale) as i32;
                let height = (size.height as f64 * scale) as i32;

                if let Some(renderer) = &mut self.renderer {
                    renderer
                        .resize(Vec2f::new(width as f32, height as f32))
                        .expect("Failed to resize window");
                }
            }

            WindowEvent::RedrawRequested => {
                let gl = self.gl.as_ref().unwrap();

                let dt = self.renderer.as_ref().unwrap().delta_time();
                let camera = self.renderer.as_mut().unwrap().camera_mut();

                if self.pressed_keys.contains(&KeyCode::KeyW) {
                    camera.move_towards(Vec3f::new(0.0, 0.0, -1.0) * dt);
                }

                if self.pressed_keys.contains(&KeyCode::KeyS) {
                    camera.move_towards(Vec3f::new(0.0, 0.0, 1.0) * dt);
                }

                if self.pressed_keys.contains(&KeyCode::KeyA) {
                    camera.move_towards(Vec3f::new(1.0, 0.0, 0.0) * dt);
                }

                if self.pressed_keys.contains(&KeyCode::KeyD) {
                    camera.move_towards(Vec3f::new(-1.0, 0.0, 0.0) * dt);
                }

                if self.pressed_keys.contains(&KeyCode::KeyQ) {
                    camera.move_towards(Vec3f::new(0.0, -1.0, 0.0) * dt);
                }

                if self.pressed_keys.contains(&KeyCode::KeyE) {
                    camera.move_towards(Vec3f::new(0.0, 1.0, 0.0) * dt);
                }

                if self.pressed_keys.contains(&KeyCode::ArrowUp) {
                    camera.rotate(Vec2f::new(0.0, 1.0) * self.camera_sensitivity * dt);
                }

                if self.pressed_keys.contains(&KeyCode::ArrowDown) {
                    camera.rotate(Vec2f::new(0.0, -1.0) * self.camera_sensitivity * dt);
                }

                if self.pressed_keys.contains(&KeyCode::ArrowLeft) {
                    camera.rotate(Vec2f::new(1.0, 0.0) * self.camera_sensitivity * dt);
                }

                if self.pressed_keys.contains(&KeyCode::ArrowRight) {
                    camera.rotate(Vec2f::new(-1.0, 0.0) * self.camera_sensitivity * dt);
                }

                let raw_input = self
                    .egui_state
                    .as_mut()
                    .unwrap()
                    .take_egui_input(self.window.as_ref().unwrap());

                let full_output = self.egui_ctx.run_ui(raw_input, |ctx| {
                    egui::Window::new("Config").show(ctx, |ui| {
                        self.ui_state
                            .render_config(ui, self.renderer.as_mut().unwrap());
                    });
                });
                {
                    self.ui_state
                        .update_app(&mut self.camera_sensitivity, &mut self.camera_speed);
                }
                self.ui_state
                    .update_renderer(self.renderer.as_mut().unwrap());

                self.egui_state.as_mut().unwrap().handle_platform_output(
                    self.window.as_ref().unwrap(),
                    full_output.platform_output,
                );

                let clipped_primitives = self
                    .egui_ctx
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                unsafe {
                    gl.clear_color(0.1, 0.2, 0.3, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);

                    self.renderer.as_mut().unwrap().render();

                    self.gl
                        .as_ref()
                        .unwrap()
                        .bind_framebuffer(glow::FRAMEBUFFER, None);

                    self.gl.as_ref().unwrap().disable(glow::BLEND);
                    self.gl.as_ref().unwrap().disable(glow::DEPTH_TEST);
                    self.gl.as_ref().unwrap().disable(glow::SCISSOR_TEST);
                    self.gl.as_ref().unwrap().color_mask(true, true, true, true);
                    self.painter.as_mut().unwrap().paint_and_update_textures(
                        [
                            self.window.as_ref().unwrap().inner_size().width,
                            self.window.as_ref().unwrap().inner_size().height,
                        ],
                        full_output.pixels_per_point,
                        &clipped_primitives,
                        &full_output.textures_delta,
                    );
                }

                self.surface
                    .as_ref()
                    .unwrap()
                    .swap_buffers(self.context.as_ref().unwrap())
                    .unwrap();
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(renderer) = &mut self.renderer {
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => {
                            let new_speed =
                                renderer.camera_mut().speed - y * 0.01 * self.camera_speed;
                            renderer.camera_mut().set_speed(new_speed);
                        }
                        winit::event::MouseScrollDelta::PixelDelta(pos) => {
                            let new_speed = renderer.camera_mut().speed
                                - pos.y as f32 * 0.01 * self.camera_speed;
                            renderer.camera_mut().set_speed(new_speed);
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.pressed_keys.insert(key);
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
