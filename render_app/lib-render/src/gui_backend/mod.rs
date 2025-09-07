use std::collections::HashMap;

use winit::dpi::PhysicalSize;

use crate::types::{GeometryType, Instance, Vertex};

pub struct BackendGraphicsInterface {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    index_count: usize,
    instance_count: usize,
}

impl BackendGraphicsInterface {
    pub(crate) fn interpret_stage(stage: HashMap<GeometryType, Vec<Instance>>, window_size: PhysicalSize<u32>) -> (Vec<Vertex>, Vec<u32>, Vec<Instance>) {
        for (geometry, elements) in stage {
            let mut instances: Vec<Instance> = Vec::new();


            for mut instance in elements {
                instance.position = [instance.position[0] * window_size.width as f32, instance.position[1] * window_size.height as f32];
                instance.scale = [instance.scale[0] * window_size.width as f32, instance.scale[1] * window_size.height as f32];
                instances.push(instance);
            }
            match geometry {
                GeometryType::Quadrilateral => {
                    let indices = [0, 1, 2, 2, 3, 0].to_vec();
                    let vertices = [
                        Vertex {
                            position: [-0.5, -0.5, 0.0], // Bottom-left
                            color: [1.0, 0.0, 0.0, 1.0],
                        },
                        Vertex {
                            position: [0.5, -0.5, 0.0], // Bottom-right
                            color: [1.0, 0.0, 0.0, 1.0],
                        },
                        Vertex {
                            position: [0.5, 0.5, 0.0],  // Top-right
                            color: [1.0, 0.0, 0.0, 1.0],
                        },
                        Vertex {
                            position: [-0.5, 0.5, 0.0], // Top-left
                            color: [1.0, 0.0, 0.0, 1.0],
                        }
                    ].to_vec();
                    return (vertices, indices, instances)
                }
            }
        }
        (Vec::new(), Vec::new(), Vec::new())
    }

    pub fn initialize_buffers(device: &wgpu::Device, vertex_count: usize, index_count: usize, instance_count: usize) -> Self {
        let vertex_buffer_size = (vertex_count * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress;
        let index_buffer_size = (index_count * std::mem::size_of::<u32>()) as wgpu::BufferAddress;
        let instance_buffer_size = (instance_count * std::mem::size_of::<Instance>()) as wgpu::BufferAddress;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: vertex_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: index_buffer_size,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: instance_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            index_count,
            instance_count,
        }
    }

    pub fn update_buffer_data(&mut self, queue: &wgpu::Queue, vertices: &[Vertex], indices: &[u32], instances: &[Instance]) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if self.vertex_buffer.size() == 0 {
            return;
        } else {
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.index_count as u32, 0, 0..self.instance_count as u32);
        }
    }
}