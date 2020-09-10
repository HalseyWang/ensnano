use crate::{camera, instance, light, mesh, texture, uniforms, utils};
use camera::{Camera, Projection};
use iced_wgpu::wgpu;
use instance::{Instance, InstanceRaw};
use light::create_light;
use mesh::{DrawModel, Mesh, Vertex};
use texture::Texture;
use uniforms::Uniforms;
use utils::create_buffer_with_data;
use wgpu::{BindGroup, BindGroupLayout, Device, RenderPass, RenderPipeline};

/// A structure that can create a pipeline which will draw several instances of the same
/// mesh.
pub struct PipelineHandler {
    mesh: Mesh,
    instances: Vec<Instance>,
    viewer_data: Uniforms,
    bind_groups: BindGroups,
    vertex_module: wgpu::ShaderModule,
    fragment_module: wgpu::ShaderModule,
    primitive_topology: wgpu::PrimitiveTopology,
}

impl PipelineHandler {
    pub fn new(
        device: &Device,
        mesh: Mesh,
        instances: Vec<Instance>,
        camera: &Camera,
        projection: &Projection,
        primitive_topology: wgpu::PrimitiveTopology,
        fake_color: bool,
    ) -> Self {
        let instances_data: Vec<_> = instances.iter().map(|i| i.to_raw()).collect();
        let (instances_bg, instances_layout) = create_instances_bind_group(device, &instances_data);

        let mut viewer_data = Uniforms::new();
        viewer_data.update_view_proj(camera, projection);
        let (viewer, viewer_layout) = create_viewer_bind_group(device, &viewer_data);

        let (light, light_layout) = create_light(device);

        let bind_groups = BindGroups {
            instances: instances_bg,
            instances_layout,
            viewer,
            viewer_layout,
            light,
            light_layout,
        };

        let vs = include_bytes!("vert.spv");
        let fs = include_bytes!("frag.spv");
        let fake_fs = include_bytes!("fake_color.spv");

        let vertex_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        let fragment_module = if fake_color {
            device.create_shader_module(
                &wgpu::read_spirv(std::io::Cursor::new(&fake_fs[..])).unwrap(),
            )
        } else {
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap())
        };

        Self {
            mesh,
            instances,
            viewer_data,
            bind_groups,
            vertex_module,
            fragment_module,
            primitive_topology,
        }
    }

    pub fn update_viewer(&mut self, device: &Device, camera: &Camera, projection: &Projection) {
        self.viewer_data.update_view_proj(camera, projection);
        let (viewer, viewer_layout) = create_viewer_bind_group(device, &self.viewer_data);
        self.bind_groups.viewer = viewer;
        self.bind_groups.viewer_layout = viewer_layout;
    }

    pub fn update_instances(&mut self, device: &Device, instances: Vec<Instance>) {
        self.instances = instances;
        let instances_data: Vec<_> = self.instances.iter().map(|i| i.to_raw()).collect();
        let (instances_bg, instances_layout) = create_instances_bind_group(device, &instances_data);
        self.bind_groups.instances = instances_bg;
        self.bind_groups.instances_layout = instances_layout;
    }

    pub fn draw<'a, 'b: 'a>(&'b self, device: &Device, render_pass: &mut RenderPass<'a>) {
        let pipeline = self.create_pipeline(device);
        render_pass.set_pipeline(&pipeline);

        render_pass.draw_mesh_instanced(
            &self.mesh,
            0..self.instances.len() as u32,
            &self.bind_groups.viewer,
            &self.bind_groups.instances,
            &self.bind_groups.light,
        );
    }

    fn create_pipeline(&self, device: &Device) -> RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &self.bind_groups.viewer_layout,
                    &self.bind_groups.instances_layout,
                    &self.bind_groups.light_layout,
                ],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &self.vertex_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &self.fragment_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: self.primitive_topology,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[mesh::MeshVertex::desc()],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
}

struct BindGroups {
    instances: BindGroup,
    instances_layout: BindGroupLayout,
    viewer: BindGroup,
    viewer_layout: BindGroupLayout,
    light: BindGroup,
    light_layout: BindGroupLayout,
}
/// Create the bind group for the model matrices.
fn create_instances_bind_group<I: bytemuck::Pod>(
    device: &Device,
    instances_data: &[I],
) -> (BindGroup, BindGroupLayout) {
    // create the model matrices and fill them in instance_buffer
    // instances_data has type &[InstanceRaw]
    let instance_buffer_size = instances_data.len() * std::mem::size_of::<InstanceRaw>();
    let instance_buffer = create_buffer_with_data(
        &device,
        bytemuck::cast_slice(instances_data),
        wgpu::BufferUsage::STORAGE_READ,
    );

    let instance_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::StorageBuffer {
                    // We don't plan on changing the size of this buffer
                    dynamic: false,
                    // The shader is not allowed to modify it's contents
                    readonly: true,
                },
            }],
        });

    let instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &instance_bind_group_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &instance_buffer,
                range: 0..instance_buffer_size as wgpu::BufferAddress,
            },
        }],
    });

    (instance_bind_group, instance_bind_group_layout)
}

/// Create the bind group for the perspective and view matrices.
fn create_viewer_bind_group<V: bytemuck::Pod>(
    device: &Device,
    viewer_data: &V,
) -> (BindGroup, BindGroupLayout) {
    let viewer_buffer = create_buffer_with_data(
        &device,
        bytemuck::cast_slice(&[*viewer_data]),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );
    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // perspective and view
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
        });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        bindings: &[
            // perspective and view
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &viewer_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..std::mem::size_of_val(&viewer_data) as wgpu::BufferAddress,
                },
            },
        ],
    });

    (uniform_bind_group, uniform_bind_group_layout)
}
