use std::{collections::HashMap, iter, sync::Arc};

pub mod types;
pub mod gui_backend;
mod camera;
mod models;
mod texture;
mod utilities;

use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize, event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window
};

use crate::{camera::{Camera2D, Camera2DUniform, Camera3D, Camera3DUniform}, gui_backend::BackendGraphicsInterface, models::{DrawModel, model, types::ModelVertex}, types::{GeometryType, Instance, Vertex}, utilities::pipeline::PipeLineBuilder};

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,

    ui_render_pipeline: wgpu::RenderPipeline,
    pub window: Arc<Window>,

    ui_camera: Camera2D,
    ui_camera_buffer: wgpu::Buffer,
    ui_camera_bind_group: wgpu::BindGroup,

    model_camera: Camera3D,
    model_camera_buffer: wgpu::Buffer,
    model_camera_bind_group: wgpu::BindGroup,

    backend_graphics_interface: BackendGraphicsInterface,
    staged_ui_data: HashMap<GeometryType, Vec<Instance>>,

    model_render_pipeline: wgpu::RenderPipeline,
    obj_model: model::Model,
    obj_model_outer: model::Model,

    rotation_angle: f32,
}

impl State {
    pub async fn new(window: Arc<Window>, staged_ui_data: HashMap<GeometryType, Vec<Instance>>) -> anyhow::Result<State> {
        let window_size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off, // Trace path
            })
            .await
            .unwrap();

        let (vertices, indices, instances) = BackendGraphicsInterface::interpret_stage(staged_ui_data.clone(), window_size);
        let mut backend_graphics_interface = BackendGraphicsInterface::initialize_buffers(&device, vertices.len(), indices.len(), instances.len());

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let initial_width = window_size.width.max(1);
        let initial_height = window_size.height.max(1);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_width,
            height: initial_height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let ui_camera = Camera2D::new(window_size.width, window_size.height);
        let camera_uniform = Camera2DUniform {
            view_proj: ui_camera.build_view_projection_matrix().to_cols_array_2d(),
        };
        let ui_camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ],
            label: Some("Camera Bind Group Layout"),
        });
        let ui_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Camera 2D Bind Group"), 
            layout: &camera_bind_group_layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ui_camera_buffer.as_entire_binding(),
                }
            ] 
        });

        let model_camera = Camera3D::new(window_size);
        let model_camera_uniform = Camera3DUniform { view_proj: model_camera.build_view_projection_matrix().to_cols_array_2d() };
        let model_camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Model Unifiform Buffer"),
            contents: bytemuck::cast_slice(&[model_camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });
        let model_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera 3D Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_camera_buffer.as_entire_binding(),
                }
            ]
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let obj_model = model::load_model("a.obj", &device, &queue, &texture_bind_group_layout, 0.99, [1.0, 0.0, 0.0, 0.5]).await.unwrap();
        let obj_model_outer = model::load_model("a.obj", &device, &queue, &texture_bind_group_layout, 1.0, [0.0, 1.0, 0.0, 0.5]).await.unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ui_shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                    Instance::desc()
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always, // No depth test
                stencil: wgpu::StencilState::default(), // No stencil operations
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        });

        let ui_render_pipeline = PipeLineBuilder::new(&device)
            .set_shader_module("ui_shader.wgsl", "vs_main", "fs_main")
            .add_bind_group_layout(&camera_bind_group_layout)
            .add_vertex_buffer_layout(Vertex::desc())
            .add_vertex_buffer_layout(Instance::desc())
            .set_pixel_format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .build("UI Render Pipeline").await;

        let model_render_pipeline = PipeLineBuilder::new(&device)
            .set_shader_module("model_shader.wgsl", "vs_main", "fs_main")
            .add_bind_group_layout(&texture_bind_group_layout)
            .add_bind_group_layout(&camera_bind_group_layout)
            .add_vertex_buffer_layout(ModelVertex::desc())
            .set_pixel_format(wgpu::TextureFormat::Rgba8UnormSrgb)
            .build("Model Render Pipeline").await;

        backend_graphics_interface.update_buffer_data(&queue, &vertices, &indices, &instances);
        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            ui_render_pipeline,
            window,

            ui_camera,
            ui_camera_buffer,
            ui_camera_bind_group,

            model_camera,
            model_camera_buffer,
            model_camera_bind_group,

            backend_graphics_interface,
            staged_ui_data,

            model_render_pipeline,

            obj_model,
            obj_model_outer,

            rotation_angle: 0.0,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.ui_camera.update_screen_size(PhysicalSize::new(width, height));
            self.queue.write_buffer(&self.ui_camera_buffer, 0, bytemuck::cast_slice(&[Camera2DUniform {
                view_proj: self.ui_camera.build_view_projection_matrix().to_cols_array_2d(),
            }]));

            self.model_camera.update_screen_size(PhysicalSize::new(width, height));
            self.queue.write_buffer(&self.model_camera_buffer, 0, bytemuck::cast_slice(&[Camera3DUniform {
                view_proj: self.model_camera.build_view_projection_matrix().to_cols_array_2d(),
            }]));

            let (vertices, indices, instances) = BackendGraphicsInterface::interpret_stage(self.staged_ui_data.clone(), PhysicalSize::new(width, height));
            self.backend_graphics_interface.update_buffer_data(&self.queue, &vertices, &indices, &instances);
        }
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
        match (key, pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn update(&mut self) {
        // Define how fast the camera should spin.
        // Smaller values are slower.
        const ROTATION_SPEED: f32 = 0.005;

        // 1. Increment the angle for the next frame.
        self.rotation_angle += ROTATION_SPEED;

        // 2. Calculate the new position using sin and cos.
        // We get the radius from the camera's initial Z position.
        let radius = 5.0; 
        let new_x = radius * self.rotation_angle.cos();
        let new_z = radius * self.rotation_angle.sin();

        // 3. Update the camera's position.
        // We only change x and z to orbit horizontally.
        self.model_camera.position.x = new_x;
        self.model_camera.position.z = new_z;

        // 4. IMPORTANT: Update the camera's uniform buffer on the GPU.
        // This sends the new camera data to your shader.
        self.queue.write_buffer(
            &self.model_camera_buffer, 
            0, 
            bytemuck::cast_slice(&[Camera3DUniform {
                view_proj: self.model_camera.build_view_projection_matrix().to_cols_array_2d(),
            }])
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    // ATTACH THE STENCIL BUFFER HERE
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                // Your existing UI render call is fine
                render_pass.set_pipeline(&self.ui_render_pipeline);
                render_pass.set_bind_group(0, &self.ui_camera_bind_group, &[]);
                self.backend_graphics_interface.render(&mut render_pass);

                // --- NEW 3-PASS DRAWING LOGIC ---

                // 1. Draw green 'A' to create the stencil mask (no color is written)
                //render_pass.set_pipeline(&self.stencil_mask_pipeline);
                //render_pass.set_stencil_reference(1);
                //render_pass.draw_model(&self.obj_model_outer, &self.model_camera_bind_group);

                // 2. Draw red 'A', using the stencil mask to clip it
                //render_pass.set_pipeline(&self.stencil_draw_pipeline);
                //render_pass.set_stencil_reference(1);
                //render_pass.draw_model(&self.obj_model, &self.model_camera_bind_group);
                
                // 3. Draw the green 'A' normally so it's visible
                render_pass.set_pipeline(&self.model_render_pipeline);
                render_pass.draw_model(&self.obj_model_outer, &self.model_camera_bind_group);
            }


        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}