// // main.rs
//
// use winit::{
//     event::*,
//     event_loop::{ControlFlow, EventLoop},
//     window::{Window, WindowBuilder},
// };
//
// use wgpu::util::DeviceExt;
//
// async fn run() {
//     // Initialize winit and create a window
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop);
//
//     // Initialize wgpu
//     let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
//     let surface = unsafe { instance.create_surface(&window) };
//     let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
//         power_preference: wgpu::PowerPreference::Default,
//         compatible_surface: Some(&surface),
//     }).await.unwrap();
//     let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();
//
//     // Create a triangle vertex buffer
//     let vertex_data = [
//         // Vertex positions (x, y)
//         0.0, 0.5,
//         -0.5, -0.5,
//         0.5, -0.5,
//     ];
//     let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//         label: Some("Vertex Buffer"),
//         contents: bytemuck::cast_slice(&vertex_data),
//         usage: wgpu::BufferUsage::VERTEX,
//     });
//
//     // Load shaders
//     let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
//     let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));
//
//     // Create pipeline
//     let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//         label: Some("Pipeline Layout"),
//         bind_group_layouts: &[],
//         push_constant_ranges: &[],
//     });
//     let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: Some("Render Pipeline"),
//         layout: Some(&pipeline_layout),
//         vertex: wgpu::VertexState {
//             module: &vs_module,
//             entry_point: "main",
//             buffers: &[],
//         },
//         fragment: Some(wgpu::FragmentState {
//             module: &fs_module,
//             entry_point: "main",
//             targets: &[wgpu::ColorTargetState {
//                 format: wgpu::TextureFormat::Bgra8UnormSrgb,
//                 blend: Some(wgpu::BlendState::REPLACE),
//                 write_mask: wgpu::ColorWrite::ALL,
//             }],
//         }),
//         primitive: wgpu::PrimitiveState {
//             topology: wgpu::PrimitiveTopology::TriangleList,
//             strip_index_format: None,
//             front_face: wgpu::FrontFace::Ccw,
//             cull_mode: Some(wgpu::Face::Back),
//             polygon_mode: wgpu::PolygonMode::Fill,
//             clamp_depth: false,
//             conservative: false,
//         },
//         depth_stencil: None,
//         multisample: wgpu::MultisampleState::default(),
//     });
//
//     // Main loop
//     event_loop.run(move |event, _, control_flow| {
//         match event {
//             Event::WindowEvent { event, .. } => match event {
//                 WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
//                 _ => {}
//             },
//             Event::MainEventsCleared => {
//                 // Render frame
//                 let frame = surface.get_current_frame().unwrap().output;
//                 let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
//                 {
//                     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                         label: Some("Render Pass"),
//                         color_attachments: &[wgpu::RenderPassColorAttachment {
//                             view: &frame.view,
//                             resolve_target: None,
//                             ops: wgpu::Operations {
//                                 load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
//                                 store: true,
//                             },
//                         }],
//                         depth_stencil_attachment: None,
//                     });
//                     render_pass.set_pipeline(&pipeline);
//                     render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
//                     render_pass.draw(0..3, 0..1);
//                 }
//                 queue.submit(std::iter::once(encoder.finish()));
//             }
//             _ => {}
//         }
//     });
// }
//
// fn main() {
//     env_logger::init();
//     wgpu_subscriber::initialize_default_subscriber(None);
//     pollster::block_on(run());
// }
