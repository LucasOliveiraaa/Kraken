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
use kmath::Vec2f;
use krender::Renderer;
use raw_window_handle::HasWindowHandle;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;
use winit::window::WindowAttributes;

pub struct App {
    renderer: Option<Renderer>,

    painter: Option<Painter>,
    egui_state: Option<State>,
    egui_ctx: EguiContext,

    gl: Option<Arc<Context>>,
    context: Option<PossiblyCurrentContext>,
    surface: Option<Surface<WindowSurface>>,
    window: Option<Window>,
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,

            window: None,
            gl: None,
            context: None,
            surface: None,

            egui_ctx: EguiContext::default(),
            egui_state: None,
            painter: None,
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
        self.renderer = Some(Renderer::new(
            self.gl.as_ref().unwrap().clone(),
            Vec2f::new(
                self.window.as_ref().unwrap().inner_size().width as f32,
                self.window.as_ref().unwrap().inner_size().height as f32,
            ),
        ));
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

                let raw_input = self
                    .egui_state
                    .as_mut()
                    .unwrap()
                    .take_egui_input(self.window.as_ref().unwrap());

                let full_output = self.egui_ctx.run_ui(raw_input, |ctx| {
                    egui::Window::new("Hello").show(ctx, |ui| {
                        ui.label("Hello from egui!");
                        ui.button("Click me");
                    });
                });

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

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
