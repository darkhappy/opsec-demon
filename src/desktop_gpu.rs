//! Vulkan resources shared by native presentation and the 4K offscreen smoke.
use eframe::{egui, egui_wgpu};
use egui_wgpu::wgpu;

struct Resources {
    pipeline: wgpu::RenderPipeline,
    group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
}
pub fn install(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    renderer: &mut egui_wgpu::Renderer,
) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Neon concert shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../assets/desktop.wgsl").into()),
    });
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Concert controls"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Vulkan concert background"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview: None,
        cache: None,
    });
    renderer.callback_resources.insert(Resources {
        pipeline,
        group,
        uniform,
    });
}
pub struct Backdrop {
    pub time: f32,
    pub aspect: f32,
    pub burst: f32,
    pub page: f32,
}
impl egui_wgpu::CallbackTrait for Backdrop {
    fn prepare(
        &self,
        _: &wgpu::Device,
        queue: &wgpu::Queue,
        _: &egui_wgpu::ScreenDescriptor,
        _: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let r = resources
            .get::<Resources>()
            .expect("installed concert resources");
        let bytes: Vec<u8> = [self.time, self.aspect, self.burst, self.page]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        queue.write_buffer(&r.uniform, 0, &bytes);
        Vec::new()
    }
    fn paint(
        &self,
        _: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let r = resources
            .get::<Resources>()
            .expect("installed concert resources");
        pass.set_pipeline(&r.pipeline);
        pass.set_bind_group(0, &r.group, &[]);
        pass.draw(0..3, 0..1);
    }
}

pub fn configuration(uncapped: bool) -> egui_wgpu::WgpuConfiguration {
    let mut setup = egui_wgpu::WgpuSetupCreateNew::default();
    setup.instance_descriptor.backends = wgpu::Backends::VULKAN;
    setup.power_preference = wgpu::PowerPreference::HighPerformance;
    egui_wgpu::WgpuConfiguration {
        wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(setup),
        present_mode: if uncapped {
            wgpu::PresentMode::AutoNoVsync
        } else {
            wgpu::PresentMode::AutoVsync
        },
        ..Default::default()
    }
}

/// Render every page on a real Vulkan adapter, read back pixels, then benchmark submissions.
pub fn smoke(shell: crate::Shell) -> std::io::Result<u8> {
    use std::{
        io,
        time::{Duration, Instant},
    };
    let fail = |error: &dyn std::fmt::Display| io::Error::other(error.to_string());
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .map_err(|e| fail(&e))?;
    let info = adapter.get_info();
    if info.backend != wgpu::Backend::Vulkan {
        return Err(io::Error::other("Smoke requires Vulkan"));
    }
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|e| fail(&e))?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut renderer = egui_wgpu::Renderer::new(&device, format, Default::default());
    install(&device, format, &mut renderer);
    let context = egui::Context::default();
    crate::desktop::prepare_smoke(&context);
    context.set_pixels_per_point(2.);
    let mut deck = crate::desktop::Deck::new(shell, format!("Vulkan / {}", info.name), false);
    deck.send("echo VULKAN_NATIVE_OK");
    let deadline = Instant::now() + Duration::from_secs(15);
    while !deck.native_smoke_ready() {
        deck.drain();
        if Instant::now() >= deadline {
            return Err(io::Error::other(format!(
                "Graphical worker failed native command smoke: {}",
                deck.smoke_diagnostics()
            )));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    for command in [
        "band on",
        "band drummer explode",
        "pedal stomp",
        "ssd explode",
        "sdvx hit bt-a",
        "novel start",
        "discord preview",
    ] {
        deck.send(command);
    }
    while !deck.smoke_ready() {
        deck.drain();
        if Instant::now() >= deadline {
            return Err(io::Error::other(
                "Graphical worker failed native command / model smoke",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    std::fs::create_dir_all("target")?;
    let size = wgpu::Extent3d {
        width: 3840,
        height: 2160,
        depth_or_array_layers: 1,
    };
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("4K smoke frame"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("4K pixel readback"),
        size: u64::from(size.width) * u64::from(size.height) * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let screen = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [3840, 2160],
        pixels_per_point: 2.,
    };
    for capture in 0..=crate::desktop::PAGES.len() {
        let source = capture == crate::desktop::PAGES.len();
        let page = if source { 2 } else { capture };
        crate::desktop::set_page(&mut deck, page);
        crate::desktop::set_source_view(&mut deck, source);
        // First pass initializes layout / fonts. Second pass is the settled screenshot.
        for settled in 0..2 {
            let output = context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1920., 1080.),
                    )),
                    time: Some(capture as f64),
                    ..Default::default()
                },
                |ctx| deck.ui(ctx),
            );
            let jobs = context.tessellate(output.shapes, 2.);
            for (id, delta) in &output.textures_delta.set {
                renderer.update_texture(&device, &queue, *id, delta);
            }
            let mut encoder = device.create_command_encoder(&Default::default());
            let buffers = renderer.update_buffers(&device, &queue, &mut encoder, &jobs, &screen);
            {
                let mut pass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    })
                    .forget_lifetime();
                renderer.render(&mut pass, &jobs, &screen);
            }
            if settled == 1 {
                encoder.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                        texture: &target,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                        buffer: &readback,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(3840 * 4),
                            rows_per_image: Some(2160),
                        },
                    },
                    size,
                );
            }
            queue.submit(buffers.into_iter().chain([encoder.finish()]));
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(15)),
                })
                .map_err(|e| fail(&e))?;
            for id in &output.textures_delta.free {
                renderer.free_texture(id);
            }
            if settled == 1 {
                let (tx, rx) = std::sync::mpsc::channel();
                readback
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        let _ = tx.send(result);
                    });
                device
                    .poll(wgpu::PollType::Wait {
                        submission_index: None,
                        timeout: Some(Duration::from_secs(15)),
                    })
                    .map_err(|e| fail(&e))?;
                rx.recv_timeout(Duration::from_secs(15))
                    .map_err(|e| fail(&e))?
                    .map_err(|e| fail(&e))?;
                let pixels = readback.slice(..).get_mapped_range();
                if !pixels
                    .chunks_exact(4)
                    .any(|p| p[0] > 30 || p[1] > 30 || p[2] > 30)
                {
                    return Err(io::Error::other("Vulkan returned an empty frame"));
                }
                let filename = if source {
                    "target/desktop-audio-source-4k.png".into()
                } else if page == 0 {
                    "target/desktop-4k.png".into()
                } else {
                    format!("target/desktop-page-{page}-4k.png")
                };
                let mut png = png::Encoder::new(std::fs::File::create(filename)?, 3840, 2160);
                png.set_color(png::ColorType::Rgba);
                png.set_depth(png::BitDepth::Eight);
                png.write_header()?.write_image_data(&pixels)?;
                drop(pixels);
                readback.unmap();
            }
        }
    }
    // GPU render-throughput experiment: same concert UI, no readback / PNG / window VSync.
    crate::desktop::set_page(&mut deck, 1);
    let output = context.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1920., 1080.),
            )),
            ..Default::default()
        },
        |ctx| deck.ui(ctx),
    );
    let jobs = context.tessellate(output.shapes, 2.);
    for (id, delta) in &output.textures_delta.set {
        renderer.update_texture(&device, &queue, *id, delta);
    }
    let began = Instant::now();
    let frames = 120;
    for _ in 0..frames {
        let mut encoder = device.create_command_encoder(&Default::default());
        let buffers = renderer.update_buffers(&device, &queue, &mut encoder, &jobs, &screen);
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();
            renderer.render(&mut pass, &jobs, &screen);
        }
        queue.submit(buffers.into_iter().chain([encoder.finish()]));
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(15)),
            })
            .map_err(|e| fail(&e))?;
    }
    let seconds = began.elapsed().as_secs_f64();
    let report = serde_json::json!({"backend":"Vulkan","adapter":info.name,"resolution":[3840,2160],"pages_rendered":7,"frames":frames,"seconds":seconds,"offscreen_render_fps":f64::from(frames)/seconds,"scope":"CPU submission + GPU completion, cached UI, no display/VSync, not interactive FPS","native_command":"passed","effects_and_graphs":"shared model state verified"});
    std::fs::write(
        "target/vulkan-smoke.json",
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "[VULKAN] 7 pages at 3840x2160 / {} / {:.1} offscreen render fps (not display FPS)",
        info.name,
        f64::from(frames) / seconds
    );
    Ok(0)
}
