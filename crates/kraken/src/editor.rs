use std::{cell::RefCell, collections::HashSet, rc::Rc};

use kmath::Vec2f;
use winit::keyboard::KeyCode;

use crate::{egui::EguiState, viewport::Viewport};

pub enum WindowEvent {
    KeyPress(KeyCode),
    KeyRelease(KeyCode),
    MouseMove(Vec2f),
    MouseState {
        left: bool,
        middle: bool,
        right: bool,
    },
    MouseScroll(Vec2f),
}

pub struct MouseState {
    pub position: Vec2f,
    pub left_button_pressed: bool,
    pub middle_button_pressed: bool,
    pub right_button_pressed: bool,

    pub scroll_delta: Vec2f,
}

pub struct Editor {
    egui_state: Rc<RefCell<EguiState>>,
    viewport: Rc<RefCell<Viewport>>,

    pressed_keys: HashSet<KeyCode>,
    mouse_state: MouseState,
}

impl Editor {
    pub fn new(egui_state: Rc<RefCell<EguiState>>, viewport: Rc<RefCell<Viewport>>) -> Self {
        Self {
            egui_state,
            viewport,

            pressed_keys: HashSet::new(),
            mouse_state: MouseState {
                position: Vec2f::new(0.0, 0.0),
                left_button_pressed: false,
                middle_button_pressed: false,
                right_button_pressed: false,
                scroll_delta: Vec2f::new(0.0, 0.0),
            },
        }
    }

    pub fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }

    pub fn mouse_state(&self) -> &MouseState {
        &self.mouse_state
    }

    pub fn resize(&mut self, resolution: Vec2f) -> Result<(), String> {
        let mut viewport = self.viewport.borrow_mut();
        viewport.resize(resolution)
    }

    pub fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyPress(key) => {
                self.pressed_keys.insert(key);
            }
            WindowEvent::KeyRelease(key) => {
                self.pressed_keys.remove(&key);
            }
            WindowEvent::MouseMove(position) => {
                self.mouse_state.position = position;
            }
            WindowEvent::MouseState {
                left,
                middle,
                right,
            } => {
                self.mouse_state.left_button_pressed = left;
                self.mouse_state.middle_button_pressed = middle;
                self.mouse_state.right_button_pressed = right;
            }
            WindowEvent::MouseScroll(delta) => {
                self.mouse_state.scroll_delta = delta;
            }
        }
    }

    pub fn egui_consume_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        let mut egui_state = self.egui_state.borrow_mut();
        egui_state.consume_window_event(window, event)
    }

    pub fn update(&mut self, window: &winit::window::Window) {
        {
            let mut viewport = self.viewport.borrow_mut();
            viewport.update(self.pressed_keys(), self.mouse_state());
        }
        {
            let mut egui_state = self.egui_state.borrow_mut();
            egui_state.update(window);
        }
    }
}
