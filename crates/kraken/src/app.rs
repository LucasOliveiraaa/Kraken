use glutin::config::ConfigTemplateBuilder;
use glutin::context::PossiblyCurrentContext;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SurfaceAttributesBuilder;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::DisplayBuilder;
use gtw::Gpu;
use kmath::Vec2f;
use raw_window_handle::HasWindowHandle;
use std::cell::RefCell;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::keyboard::PhysicalKey;
use winit::window::Window;
use winit::window::WindowAttributes;

use crate::editor::{Editor, WindowEvent};
use crate::egui::EguiState;
use crate::viewport::Viewport;

pub struct App {
    editor: Option<Editor>,

    gpu: Option<Arc<Gpu>>,

    context: Option<PossiblyCurrentContext>,
    surface: Option<Surface<WindowSurface>>,
    window: Option<Window>,
}

impl App {
    pub fn new() -> Self {
        Self {
            editor: None,

            gpu: None,

            context: None,
            surface: None,
            window: None,
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
        let gpu = Arc::new(Gpu::new(gl));

        let viewport = Viewport::new(
            gpu.clone(),
            Vec2f::new(
                window.inner_size().width as f32,
                window.inner_size().height as f32,
            ),
        )
        .expect("Failed creating viewport");
        let viewport = Rc::new(RefCell::new(viewport));

        let egui_state = EguiState::new(gpu.clone(), event_loop, viewport.clone());
        let egui_state = Rc::new(RefCell::new(egui_state));

        self.editor = Some(Editor::new(egui_state, viewport));

        self.gpu = Some(gpu);

        self.window = Some(window);
        self.surface = Some(surface);
        self.context = Some(context);
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

        let Some(editor) = &mut self.editor else {
            if matches!(event, winit::event::WindowEvent::CloseRequested) {
                event_loop.exit();
            }

            return;
        };

        if editor.egui_consume_window_event(self.window.as_ref().unwrap(), &event) {
            return;
        }

        match event {
            winit::event::WindowEvent::Resized(size) => {
                self.surface.as_ref().unwrap().resize(
                    self.context.as_ref().unwrap(),
                    NonZeroU32::new(size.width.max(1)).unwrap(),
                    NonZeroU32::new(size.height.max(1)).unwrap(),
                );

                let scale = self.window.as_ref().unwrap().scale_factor() as f32;

                let width = size.width as f32 * scale;
                let height = size.height as f32 * scale;

                editor
                    .resize(Vec2f::new(width, height))
                    .expect("Failed to resize window");
            }

            winit::event::WindowEvent::RedrawRequested => {
                self.editor
                    .as_mut()
                    .unwrap()
                    .update(self.window.as_ref().unwrap());

                self.surface
                    .as_ref()
                    .unwrap()
                    .swap_buffers(self.context.as_ref().unwrap())
                    .unwrap();
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let pos = Vec2f::new(position.x as f32, position.y as f32);

                editor.handle_window_event(WindowEvent::MouseMove(pos));
            }

            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let old_state = editor.mouse_state();

                let left =
                    button == winit::event::MouseButton::Left && state == ElementState::Pressed;
                let middle =
                    button == winit::event::MouseButton::Middle && state == ElementState::Pressed;
                let right =
                    button == winit::event::MouseButton::Right && state == ElementState::Pressed;

                editor.handle_window_event(WindowEvent::MouseState {
                    left: old_state.left_button_pressed || left,
                    middle: old_state.middle_button_pressed || middle,
                    right: old_state.right_button_pressed || right,
                });
            }

            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    let delta = Vec2f::new(x, y);

                    editor.handle_window_event(WindowEvent::MouseScroll(delta));
                }
                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                    let delta = Vec2f::new(pos.x as f32, pos.y as f32);

                    editor.handle_window_event(WindowEvent::MouseScroll(delta));
                }
            },

            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            editor.handle_window_event(WindowEvent::KeyPress(key));
                        }
                        ElementState::Released => {
                            editor.handle_window_event(WindowEvent::KeyRelease(key));
                        }
                    }
                }
            }
            winit::event::WindowEvent::CloseRequested => {
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
