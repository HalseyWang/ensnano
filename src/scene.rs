use crate::{design, instance, utils};
use crate::{DrawArea, PhySize, WindowEvent};
use futures::executor;
use iced_wgpu::wgpu;
use iced_winit::winit;
use instance::Instance;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use ultraviolet::{Mat4, Rotor3, Vec3};
use utils::BufferDimensions;
use wgpu::{Device, Queue};
use winit::dpi::PhysicalPosition;
mod camera;
mod view;
use view::{View, ViewUpdate};
mod controller;
use controller::{Consequence, Controller};
use design::Design;
use std::path::PathBuf;

type ViewPtr = Rc<RefCell<View>>;
pub struct Scene {
    designs: Vec<Design>,
    update: SceneUpdate,
    selected_id: Option<u32>,
    selected_design: Option<u32>,
    view: ViewPtr,
    controller: Controller,
    area: DrawArea,
}

impl Scene {
    /// Create a new scene.
    /// # Argument
    ///
    /// * `device` a reference to a `Device` object. This can be seen as a socket to the GPU
    ///
    /// * `window_size` the *Physical* size of the window in which the application is displayed
    ///
    /// * `area` 
    pub fn new(device: &Device, window_size: PhySize, area: DrawArea) -> Self {
        let update = SceneUpdate::new();
        let view = Rc::new(RefCell::new(View::new(window_size, area.size, device)));
        let controller = Controller::new(view.clone(), window_size, area.size);
        Self {
            view,
            designs: Vec::new(),
            update,
            selected_id: None,
            selected_design: None,
            controller,
            area,
        }
    }

    pub fn add_design(&mut self, path: &PathBuf) {
        self.designs
            .push(Design::new_with_path(path, self.designs.len() as u32))
    }

    pub fn clear_design(&mut self, path: &PathBuf) {
        self.designs = vec![Design::new_with_path(path, 0)]
    }

    /// Input an event to the scene. Return true, if the selected object of the scene has changed
    pub fn input(
        &mut self,
        event: &WindowEvent,
        device: &Device,
        queue: &mut wgpu::Queue,
        cursor_position: PhysicalPosition<f64>,
    ) {
        let camera_can_move = self.selected_design.is_none();
        let consequence = self
            .controller
            .input(event, cursor_position, camera_can_move);
        match consequence {
            Consequence::Nothing => (),
            Consequence::CameraMoved => self.notify(SceneNotification::CameraMoved),
            Consequence::PixelSelected(clicked) => self.click_on(clicked, device, queue),
            Consequence::Translation(x, y, z) => {
                self.translate_selected_design(x, y, z);
            }
            Consequence::MovementEnded => {
                for d in self.designs.iter_mut() {
                    d.update_position();
                }
            }
            Consequence::Rotation(x, y) => {
                let cam_right = self.view.borrow().right_vec();
                let cam_up = self.view.borrow().up_vec();
                let origin = self.get_selected_position().unwrap();
                self.designs[self.selected_design.unwrap() as usize]
                    .rotate(x, y, cam_right, cam_up, origin);
            }
            Consequence::Swing(x, y) => {
                if let Some(id) = self.selected_id {
                    let pivot = self.designs[self.selected_design.unwrap() as usize]
                        .get_element_position(id)
                        .unwrap();
                    self.controller.set_pivot_point(pivot);
                    self.controller.swing(x, y);
                    self.notify(SceneNotification::CameraMoved);
                }
            }
        };
    }

    fn click_on(
        &mut self,
        clicked_pixel: PhysicalPosition<f64>,
        device: &Device,
        queue: &mut Queue,
    ) {
        let (selected_id, design_id) = self.set_selected_id(clicked_pixel, device, queue);
        if selected_id != 0xFFFFFF {
            self.selected_id = Some(selected_id);
            self.selected_design = Some(design_id);
            for i in 0..self.designs.len() {
                let arg = if i == design_id as usize {
                    Some(selected_id)
                } else {
                    None
                };
                self.designs[i].update_selection(arg);
            }
        } else {
            self.selected_id = None;
            self.selected_design = None;
        }
    }

    fn set_selected_id(
        &mut self,
        clicked_pixel: PhysicalPosition<f64>,
        device: &Device,
        queue: &mut wgpu::Queue,
    ) -> (u32, u32) {
        let size = wgpu::Extent3d {
            width: self.controller.get_window_size().width,
            height: self.controller.get_window_size().height,
            depth: 1,
        };

        let (texture, texture_view) = self.create_fake_scene_texture(device, size);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.view
            .borrow_mut()
            .draw(&mut encoder, &texture_view, device, true, queue, self.area);

        // create a buffer and fill it with the texture
        let buffer_dimensions = BufferDimensions::new(size.width as usize, size.height as usize);
        let buf_size = buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height;
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: buf_size as u64,
            usage: wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
            label: Some("staging_buffer"),
        });
        let buffer_copy_view = wgpu::BufferCopyView {
            buffer: &staging_buffer,
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: buffer_dimensions.padded_bytes_per_row as u32,
                rows_per_image: 0,
            },
        };
        let texture_copy_view = wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        };
        encoder.copy_texture_to_buffer(texture_copy_view, buffer_copy_view, size);
        queue.submit(Some(encoder.finish()));

        // recover the desired pixel
        let pixel = (self.area.position.y as usize + clicked_pixel.y as usize)
            * buffer_dimensions.padded_bytes_per_row
            + (self.area.position.x as usize + clicked_pixel.x as usize)
                * std::mem::size_of::<u32>();

        let buffer_slice = staging_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
        device.poll(wgpu::Maintain::Wait);

        let future_color = async {
            if let Ok(()) = buffer_future.await {
                let pixels = buffer_slice.get_mapped_range();
                let a = pixels[pixel + 3] as u32;
                let r = (pixels[pixel + 2] as u32) << 16;
                let g = (pixels[pixel + 1] as u32) << 8;
                let b = pixels[pixel] as u32;
                let color = r + g + b;
                drop(pixels);
                staging_buffer.unmap();
                (color, a)
            } else {
                panic!("could not read fake texture");
            }
        };
        executor::block_on(future_color)
    }

    fn create_fake_scene_texture(&self, device: &Device, size: wgpu::Extent3d) -> (wgpu::Texture, wgpu::TextureView) {
        let desc = wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC,
            label: Some("desc"),
        };
        let texture_view_descriptor = wgpu::TextureViewDescriptor {
            label: Some("texture_view_descriptor"),
            format: Some(wgpu::TextureFormat::Bgra8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&texture_view_descriptor);
        (texture, view)
    }

    fn translate_selected_design(&mut self, x: f64, y: f64, z: f64) {
        let distance = (self.get_selected_position().unwrap() - self.camera_position())
            .dot(self.camera_direction())
            .abs()
            .sqrt();
        let height = 2. * distance * (self.get_fovy() / 2.).tan();
        let width = height * self.get_ratio();
        let right_vec = width * x as f32 * self.view.borrow().right_vec();
        let up_vec = height * -y as f32 * self.view.borrow().up_vec();
        let forward = z as f32 * self.view.borrow().get_camera_direction();
        self.designs[self.selected_design.expect("no design selected") as usize]
            .translate(right_vec, up_vec, forward);
    }

    fn get_selected_position(&self) -> Option<Vec3> {
        if let Some(d_id) = self.selected_design {
            self.designs[d_id as usize].get_element_position(self.selected_id.unwrap())
        } else {
            None
        }
    }

    pub fn fit_design(&mut self) {
        if self.designs.len() > 0 {
            let (position, rotor) = self.designs[0].fit(self.get_fovy(), self.get_ratio());
            self.controller
                .set_pivot_point(self.designs[0].middle_point());
            self.notify(SceneNotification::NewCamera(position, rotor));
        }
    }

    fn camera_position(&self) -> Vec3 {
        self.view.borrow().get_camera_position()
    }

    fn camera_direction(&self) -> Vec3 {
        self.view.borrow().get_camera_position()
    }

    pub fn draw_view(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        device: &Device,
        dt: Duration,
        fake_color: bool,
        queue: &Queue,
    ) {
        if self.controller.camera_is_moving() {
            self.notify(SceneNotification::CameraMoved);
        }
        self.fetch_data_updates();
        self.fetch_view_updates();
        if self.update.need_update {
            self.perform_update(dt);
        }
        self.view
            .borrow_mut()
            .draw(encoder, target, device, fake_color, queue, self.area);
    }

    fn perform_update(&mut self, dt: Duration) {
        if let Some(instance) = self.update.sphere_instances.take() {
            self.view.borrow_mut().update(ViewUpdate::Spheres(instance))
        }
        if let Some(instance) = self.update.tube_instances.take() {
            self.view.borrow_mut().update(ViewUpdate::Tubes(instance))
        }
        if let Some(sphere) = self.update.selected_sphere.take() {
            self.view
                .borrow_mut()
                .update(ViewUpdate::SelectedSpheres(vec![sphere]))
        }
        if let Some(tubes) = self.update.selected_tube.take() {
            self.view
                .borrow_mut()
                .update(ViewUpdate::SelectedTubes(tubes))
        }
        if let Some(matrices) = self.update.model_matrices.take() {
            self.view
                .borrow_mut()
                .update(ViewUpdate::ModelMatricies(matrices))
        }

        if self.update.camera_update {
            self.controller.update_camera(dt);
            self.view.borrow_mut().update(ViewUpdate::Camera);
            self.update.camera_update = false;
        }
        self.update.need_update = false;
    }

    fn fetch_data_updates(&mut self) {
        let need_update = self
            .designs
            .iter_mut()
            .fold(false, |acc, design| acc | design.data_was_updated());

        if need_update {
            let mut sphere_instances = vec![];
            let mut tube_instances = vec![];
            let mut selected_sphere_instances = vec![];
            let mut selected_tube_instances = vec![];
            for d in self.designs.iter() {
                for s in d.spheres().iter() {
                    sphere_instances.push(*s);
                }
                for t in d.tubes().iter() {
                    tube_instances.push(*t);
                }
                for s in d.selected_spheres().iter() {
                    selected_sphere_instances.push(*s);
                }
                for t in d.selected_tubes().iter() {
                    selected_tube_instances.push(*t);
                }
            }
            self.update.sphere_instances = Some(sphere_instances);
            self.update.tube_instances = Some(tube_instances);
            self.update.selected_tube = if selected_tube_instances.len() > 0 {
                Some(selected_tube_instances)
            } else {
                None
            };
            self.update.selected_sphere = if selected_sphere_instances.len() > 0 {
                Some(selected_sphere_instances[0])
            } else {
                None
            };
        }
        self.update.need_update |= need_update;
    }

    fn fetch_view_updates(&mut self) {
        let need_update = self
            .designs
            .iter_mut()
            .fold(false, |acc, design| acc | design.view_was_updated());

        if need_update {
            let matrices: Vec<_> = self.designs.iter().map(|d| d.model_matrix()).collect();
            self.update.model_matrices = Some(matrices);
        }
        self.update.need_update |= need_update;
    }

    pub fn get_fovy(&self) -> f32 {
        self.view.borrow().get_projection().borrow().get_fovy()
    }

    pub fn get_ratio(&self) -> f32 {
        self.view.borrow().get_projection().borrow().get_ratio()
    }
}

/// A structure that stores the element that needs to be updated in a scene
pub struct SceneUpdate {
    pub tube_instances: Option<Vec<Instance>>,
    pub sphere_instances: Option<Vec<Instance>>,
    pub fake_tube_instances: Option<Vec<Instance>>,
    pub fake_sphere_instances: Option<Vec<Instance>>,
    pub selected_tube: Option<Vec<Instance>>,
    pub selected_sphere: Option<Instance>,
    pub model_matrices: Option<Vec<Mat4>>,
    pub need_update: bool,
    pub camera_update: bool,
}

impl SceneUpdate {
    pub fn new() -> Self {
        Self {
            tube_instances: None,
            sphere_instances: None,
            fake_tube_instances: None,
            fake_sphere_instances: None,
            selected_tube: None,
            selected_sphere: None,
            need_update: false,
            camera_update: false,
            model_matrices: None,
        }
    }
}

pub enum SceneNotification {
    CameraMoved,
    NewCamera(Vec3, Rotor3),
    NewSize(PhySize, DrawArea),
}

impl Scene {
    pub fn notify(&mut self, notification: SceneNotification) {
        match notification {
            SceneNotification::NewCamera(position, projection) => {
                self.controller.teleport_camera(position, projection);
                self.update.camera_update = true;
            }
            SceneNotification::CameraMoved => self.update.camera_update = true,
            SceneNotification::NewSize(window_size, area) => {
                self.area = area;
                self.resize(window_size);
            }
        };
        self.update.need_update = true;
    }

    fn resize(&mut self, window_size: PhySize) {
        self.view.borrow_mut().update(ViewUpdate::Size(window_size));
        self.controller.resize(window_size, self.area.size);
        self.update.camera_update = true;
    }
}
