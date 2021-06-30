use std::iter;

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    // Face Front
    Vertex {
        position: [1., 1., 1.],
        tex_coords: [1., 0.],
    },
    Vertex {
        position: [1., -1., 1.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [1., -1., -1.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [1., 1., -1.],
        tex_coords: [0., 0.],
    },
    // Face Back
    Vertex {
        position: [-1., 1., 1.],
        tex_coords: [1., 0.],
    },
    Vertex {
        position: [-1., -1., 1.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [-1., -1., -1.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [-1., 1., -1.],
        tex_coords: [0., 0.],
    },
    // Face Left
    Vertex {
        position: [1., -1., 1.],
        tex_coords: [1., 0.],
    },
    Vertex {
        position: [-1., -1., 1.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [-1., -1., -1.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [1., -1., -1.],
        tex_coords: [0., 0.],
    },
    // Face Right
    Vertex {
        position: [1., 1., 1.],
        tex_coords: [1., 0.],
    },
    Vertex {
        position: [-1., 1., 1.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [-1., 1., -1.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [1., 1., -1.],
        tex_coords: [0., 0.],
    },
    // Face Up
    Vertex {
        position: [1., 1., 1.],
        tex_coords: [1., 0.],
    },
    Vertex {
        position: [-1., 1., 1.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [-1., -1., 1.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [1., -1., 1.],
        tex_coords: [0., 0.],
    },
    // Face Down
    Vertex {
        position: [1., 1., -1.],
        tex_coords: [1., 0.],
    },
    Vertex {
        position: [-1., 1., -1.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [-1., -1., -1.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [1., -1., -1.],
        tex_coords: [0., 0.],
    },
];

#[rustfmt::skip]
const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3,
    7, 6, 4, 6, 5, 4,
    8, 9, 10, 8, 10, 11,
    15, 14, 12, 14, 13, 12,
    16, 17, 18, 16, 18, 19,
    23, 22, 20, 22, 21, 20,
];

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

struct Camera {
    position: cgmath::Point3<f32>,
    angle_ground: cgmath::Rad<f32>,
    angle_up: cgmath::Rad<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.position, self.position + self.look_direction(), cgmath::Vector3::unit_z());
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }

    fn look_direction(&self) -> cgmath::Vector3<f32> {
        use cgmath::Angle;
        return (
            Angle::cos(self.angle_up) * Angle::cos(self.angle_ground),
            Angle::cos(self.angle_up) * Angle::sin(self.angle_ground),
            Angle::sin(self.angle_up),
        ).into()
    }

    fn forward_direction(&self) -> cgmath::Vector3<f32> {
        use cgmath::Angle;
        return (
            Angle::cos(self.angle_ground),
            Angle::sin(self.angle_ground),
            0.,
        ).into()
    }

    fn right_direction(&self) -> cgmath::Vector3<f32> {
        use cgmath::Angle;
        return (
            Angle::sin(self.angle_ground),
            -Angle::cos(self.angle_ground),
            0.,
        ).into()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

struct CameraController {
    speed: f32,
    angle_ground_delta: cgmath::Rad<f32>,
    angle_up_delta: cgmath::Rad<f32>,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            angle_ground_delta: cgmath::Rad(0.),
            angle_up_delta: cgmath::Rad(0.),
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        scancode,
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match scancode {
                    57 /*space*/ => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    42 /*shift*/ => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    17 /*W*/ => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    30 /*A*/ => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    31 /*S*/ => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    32 /*D*/ => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn process_device_event(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion {
                delta,
                ..
            } => {
                self.angle_ground_delta -= cgmath::Rad(delta.0 as f32);
                self.angle_up_delta     -= cgmath::Rad(delta.1 as f32);
                true
            }
            _ => false,
        }
    }

    fn update_camera(&mut self, camera: &mut Camera) {
        const ZERO: cgmath::Vector3<f32> = cgmath::Vector3{x: 0., y: 0., z: 0.};
        let direction =
            if self.is_forward_pressed  {  camera.forward_direction() } else { ZERO } +
            if self.is_backward_pressed { -camera.forward_direction() } else { ZERO } +
            if self.is_right_pressed    {  camera.right_direction  () } else { ZERO } +
            if self.is_left_pressed     { -camera.right_direction  () } else { ZERO } +
            if self.is_up_pressed       {  cgmath::Vector3::unit_z () } else { ZERO } +
            if self.is_down_pressed     { -cgmath::Vector3::unit_z () } else { ZERO }
        ;
        let magnitude = cgmath::InnerSpace::magnitude(direction);
        if magnitude > 0.001 {
            camera.position += direction / magnitude * self.speed;
        }
        camera.angle_ground += self.angle_ground_delta * 0.001;
        camera.angle_up     += self.angle_up_delta     * 0.001; 
        self.angle_ground_delta = cgmath::Rad(0.);
        self.angle_up_delta = cgmath::Rad(0.);
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    #[allow(dead_code)]
    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    // NEW!
    camera: Camera,
    camera_controller: CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = Camera {
            position: (-10.0, 2.0, 1.0).into(),
            angle_ground: cgmath::Rad(0.),
            angle_up: cgmath::Rad(0.),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let camera_controller = CameraController::new(0.2);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                clamp_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_texture,
            diffuse_bind_group,
            camera,
            camera_controller,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        self.camera.aspect = self.sc_desc.width as f32 / self.sc_desc.height as f32;
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn process_device_event(&mut self, event: &DeviceEvent) -> bool {
        self.camera_controller.process_device_event(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window));
    
    window.set_cursor_visible(false);
    window.set_cursor_grab(true);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => { 
                                window.set_cursor_grab(false);
                                window.set_cursor_visible(true);
                            },
                            _ => {}
                        },
                        WindowEvent::MouseInput {button: MouseButton::Left, ..} => {
                            window.set_cursor_grab(true);
                            window.set_cursor_visible(false);
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::DeviceEvent {
                ref event,
                ..
            } => {
                state.process_device_event(event);
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
