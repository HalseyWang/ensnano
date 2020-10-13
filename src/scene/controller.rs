use super::{camera, DataPtr, Duration, HandleDir, ViewPtr, WidgetRotationMode as RotationMode, SceneElement};
use crate::consts::*;
use crate::{PhySize, PhysicalPosition, WindowEvent};
use iced_winit::winit::event::*;
use ultraviolet::{Rotor3, Vec3};
use crate::design::StrandBuilder;

use camera::CameraController;

/// The effect that draging the mouse have
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClickMode {
    TranslateCam,
    RotateCam,
}

enum State {
    MoveCamera,
    Translate(HandleDir),
    Rotate(RotationMode),
    TogglingWidget,
    Building(StrandBuilder),
}

/// An object handling input and notification for the scene.
pub struct Controller {
    /// A pointer to the View
    view: ViewPtr,
    /// A pointer to the data
    data: DataPtr,
    /// The event that modify the camera are forwarded to the camera_controller
    camera_controller: CameraController,
    /// The postion where the user has clicked left
    last_left_clicked_position: Option<PhysicalPosition<f64>>,
    /// The postion where the user has clicked right
    last_right_clicked_position: Option<PhysicalPosition<f64>>,
    /// The position of the mouse
    mouse_position: PhysicalPosition<f64>,
    /// The size of the window
    window_size: PhySize,
    /// The size of the drawing area
    area_size: PhySize,
    /// The current modifiers
    current_modifiers: ModifiersState,
    /// The modifiers when a click was performed
    modifiers_when_clicked: ModifiersState,
    /// The effect that dragging the mouse has
    click_mode: ClickMode,
    state: State,
}

const NO_POS: PhysicalPosition<f64> = PhysicalPosition::new(f64::NAN, f64::NAN);

pub enum Consequence {
    CameraMoved,
    PixelSelected(PhysicalPosition<f64>),
    Translation(HandleDir, f64, f64),
    MovementEnded,
    Rotation(RotationMode, f64, f64),
    InitRotation(f64, f64),
    InitTranslation(f64, f64),
    Swing(f64, f64),
    Nothing,
    CursorMoved(PhysicalPosition<f64>),
    ToggleWidget,
}

impl Controller {
    pub fn new(view: ViewPtr, data: DataPtr, window_size: PhySize, area_size: PhySize) -> Self {
        let camera_controller = {
            let view = view.borrow();
            CameraController::new(4.0, 0.04, view.get_camera(), view.get_projection())
        };
        Self {
            view,
            data,
            camera_controller,
            last_left_clicked_position: None,
            last_right_clicked_position: None,
            mouse_position: PhysicalPosition::new(0., 0.),
            window_size,
            area_size,
            current_modifiers: ModifiersState::empty(),
            modifiers_when_clicked: ModifiersState::empty(),
            click_mode: ClickMode::TranslateCam,
            state: State::MoveCamera,
        }
    }

    /// Replace the camera by a new one.
    pub fn teleport_camera(&mut self, position: Vec3, rotation: Rotor3) {
        self.camera_controller.teleport_camera(position, rotation)
    }

    /// Handles input
    /// # Argument
    ///
    /// * `event` the event to be handled
    ///
    /// * `position` the position of the mouse *in the drawing area coordinates*
    pub fn input(&mut self, event: &WindowEvent, position: PhysicalPosition<f64>) -> Consequence {
        match event {
            WindowEvent::ModifiersChanged(modifiers) => {
                self.current_modifiers = *modifiers;
                Consequence::Nothing
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => match *key {
                VirtualKeyCode::T if *state == ElementState::Released => {
                    self.data.borrow_mut().toggle_selection_mode();
                    Consequence::Nothing
                }
                VirtualKeyCode::H if self.current_modifiers.shift() => {
                    self.data.borrow_mut().select_5prime();
                    Consequence::Nothing
                }
                VirtualKeyCode::L if self.current_modifiers.shift() => {
                    self.data.borrow_mut().select_3prime();
                    Consequence::Nothing
                }
                _ => {
                    if self.camera_controller.process_keyboard(*key, *state) {
                        Consequence::CameraMoved
                    } else {
                        Consequence::Nothing
                    }
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                Consequence::CameraMoved
            }
            WindowEvent::CursorLeft { .. } => {
                if self.last_left_clicked_position.is_some() {
                    self.last_left_clicked_position = None;
                    Consequence::MovementEnded
                } else if self.last_right_clicked_position.is_some() {
                    self.last_right_clicked_position = None;
                    Consequence::MovementEnded
                } else {
                    Consequence::Nothing
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => { 
                    let builder = if *state == ElementState::Pressed {
                        self.data.borrow_mut().get_strand_builder()
                    } else {
                        None
                    };
                match self.state {
                    State::MoveCamera => {
                        if let Some(builder) = builder {
                            self.state = State::Building(builder);
                            self.last_left_clicked_position = Some(self.mouse_position);
                            Consequence::Nothing
                        } else {
                            self.left_click_camera(state)
                        }
                    }
                    State::Rotate(_) => {
                        if *state == ElementState::Pressed {
                            let (x, y) = self.logical_mouse_position();
                            self.last_left_clicked_position = Some(self.mouse_position);
                            Consequence::InitRotation(x, y)
                        } else {
                            self.last_left_clicked_position = None;
                            Consequence::MovementEnded
                        }
                    }
                    State::Translate(_) => {
                        if *state == ElementState::Pressed {
                            let (x, y) = self.logical_mouse_position();
                            self.last_left_clicked_position = Some(self.mouse_position);
                            Consequence::InitTranslation(x, y)
                        } else {
                            self.last_left_clicked_position = None;
                            Consequence::MovementEnded
                        }
                    }
                    State::TogglingWidget => {
                        if *state == ElementState::Pressed {
                            Consequence::ToggleWidget
                        } else {
                            self.last_left_clicked_position = None;
                            Consequence::MovementEnded
                        }
                    }
                    State::Building(_) => {
                        println!("not building");
                        if *state == ElementState::Released {
                           self.state = State::MoveCamera;
                        }
                        self.left_click_camera(state)
                    },
                }
            },
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state,
                ..
            } => {
                let mut released = false;
                self.camera_controller.process_click(state);
                if *state == ElementState::Pressed {
                    self.last_right_clicked_position = Some(self.mouse_position);
                    self.modifiers_when_clicked = self.current_modifiers;
                    self.camera_controller.foccus();
                } else {
                    released = true;
                    self.state = State::MoveCamera;
                }
                if self.last_right_clicked_position.is_some() {
                    if released {
                        self.last_right_clicked_position = None;
                    }
                    Consequence::MovementEnded
                } else {
                    Consequence::Nothing
                }
            }
            WindowEvent::CursorMoved { .. } => {
                self.mouse_position = position;
                if let Some(clicked_position) = self.last_left_clicked_position {
                    let mouse_dx = (position.x - clicked_position.x) / self.area_size.width as f64;
                    let mouse_dy = (position.y - clicked_position.y) / self.area_size.height as f64;
                    let mouse_x = position.x / self.area_size.width as f64;
                    let mouse_y = position.y / self.area_size.height as f64;
                    match &self.state {
                        State::MoveCamera | State::TogglingWidget => {
                            self.camera_controller.process_mouse(mouse_dx, mouse_dy);
                            Consequence::CameraMoved
                        }
                        State::Translate(dir) => Consequence::Translation(*dir, mouse_x, mouse_y),
                        State::Rotate(mode) => Consequence::Rotation(*mode, mouse_x, mouse_y),
                        State::Building(_) => Consequence::Nothing,
                    }
                } else if let Some(clicked_position) = self.last_right_clicked_position {
                    let mouse_dx = (position.x - clicked_position.x) / self.area_size.width as f64;
                    let mouse_dy = (position.y - clicked_position.y) / self.area_size.height as f64;
                    Consequence::Swing(mouse_dx, mouse_dy)
                } else {
                    match self.state {
                        State::Building(_) => Consequence::Nothing,
                        _ => Consequence::CursorMoved(position),
                    }
                }
            }
            _ => Consequence::Nothing,
        }
    }

    /// True if the camera is moving and its position must be updated before next frame
    pub fn camera_is_moving(&self) -> bool {
        self.camera_controller.is_moving()
    }

    /// Set the pivot point of the camera
    pub fn set_pivot_point(&mut self, point: Vec3) {
        self.camera_controller.set_pivot_point(point)
    }

    /// Swing the camera arround its pivot point
    pub fn swing(&mut self, x: f64, y: f64) {
        self.camera_controller.swing(x, y);
    }

    /// Moves the camera according to its speed and the time elapsed since previous frame
    pub fn update_camera(&mut self, dt: Duration) {
        self.camera_controller.update_camera(dt, self.click_mode);
    }

    /// Handles a resizing of the window and/or drawing area
    pub fn resize(&mut self, window_size: PhySize, area_size: PhySize) {
        self.window_size = window_size;
        self.area_size = area_size;
        self.camera_controller.resize(area_size);
        // the view needs the window size to build a depth texture
        self.view
            .borrow_mut()
            .update(super::view::ViewUpdate::Size(window_size));
    }

    pub fn get_window_size(&self) -> PhySize {
        self.window_size
    }

    pub fn notify(&mut self, element: Option<SceneElement>) {
        if let Some(SceneElement::WidgetElement(widget_id)) = element {
            match widget_id {
                RIGHT_HANDLE_ID => self.state = State::Translate(HandleDir::Right),
                UP_HANDLE_ID => self.state = State::Translate(HandleDir::Up),
                DIR_HANDLE_ID => self.state = State::Translate(HandleDir::Dir),
                RIGHT_CIRCLE_ID => self.state = State::Rotate(RotationMode::Right),
                UP_CIRCLE_ID => self.state = State::Rotate(RotationMode::Up),
                FRONT_CIRCLE_ID => self.state = State::Rotate(RotationMode::Front),
                SPHERE_WIDGET_ID => self.state = State::TogglingWidget,
                _ => self.state = State::MoveCamera,
            }
        } else {
            self.state = State::MoveCamera
        }
    }

    fn left_click_camera(&mut self, state: &ElementState) -> Consequence {
        self.camera_controller.process_click(state);
        let mut released = false;
        if *state == ElementState::Pressed {
            self.last_left_clicked_position = Some(self.mouse_position);
            self.modifiers_when_clicked = self.current_modifiers;
        } else if position_difference(
            self.last_left_clicked_position.unwrap_or(NO_POS),
            self.mouse_position,
        ) < 5.
        {
            return Consequence::PixelSelected(self.last_left_clicked_position.take().unwrap());
        } else {
            released = true;
        }
        if self.last_left_clicked_position.is_some() {
            if released {
                self.last_left_clicked_position = None;
            }
            Consequence::MovementEnded
        } else {
            Consequence::Nothing
        }
    }

    fn logical_mouse_position(&self) -> (f64, f64) {
        (
            self.mouse_position.x / self.area_size.width as f64,
            self.mouse_position.y / self.area_size.height as f64,
        )
    }
}

fn position_difference(a: PhysicalPosition<f64>, b: PhysicalPosition<f64>) -> f64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}
