use anyhow::Context;
use image::{ImageBuffer, Rgba, RgbaImage};
use wgpu::util::DeviceExt;

use crate::schema::{self, ImageJson, Pixel};

const ENCODE_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= arrayLength(&input)) {
        return;
    }
    let packed = input[idx];
  let r = packed & 0xFFu;
  let g = (packed >> 8u) & 0xFFu;
  let b = (packed >> 16u) & 0xFFu;
  let a = (packed >> 24u) & 0xFFu;
  output[idx] = r | (g << 8u) | (b << 16u) | (a << 24u);
}
"#;

const DECODE_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= arrayLength(&input)) {
        return;
    }
    output[idx] = input[idx];
}
"#;

struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn init_gpu() -> anyhow::Result<GpuContext> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .context("no compatible GPU adapter found")?;

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("img2jx-gpu"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
        },
        None,
    ))
    .context("failed to create GPU device")?;

    Ok(GpuContext { device, queue })
}

fn run_compute(
    ctx: &GpuContext,
    shader_source: &str,
    label: &str,
    input_data: &[u32],
) -> anyhow::Result<Vec<u32>> {
    let shader = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

    let bind_group_layout =
        ctx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("{label}-bgl")),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

    let pipeline_layout = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{label}-pl")),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("{label}-pipeline")),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

    let input_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{label}-input")),
        contents: bytemuck::cast_slice(input_data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    let output_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("{label}-output")),
        size: (input_data.len() * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{label}-bg")),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{label}-encoder")),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(&format!("{label}-pass")),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let workgroups = (input_data.len() as u32).div_ceil(256);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("{label}-staging")),
        size: (input_data.len() * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(
        &output_buffer,
        0,
        &staging,
        0,
        staging.size(),
    );

    ctx.queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    ctx.device.poll(wgpu::Maintain::Wait);
    receiver
        .recv()
        .context("GPU map channel closed")?
        .context("failed to map GPU buffer")?;

    let data = slice.get_mapped_range();
    let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging.unmap();

    Ok(result)
}

pub fn encode_rows(rgba: &RgbaImage) -> anyhow::Result<Vec<Vec<Pixel>>> {
    let (width, height) = rgba.dimensions();
    let pixels: Vec<u32> = rgba
        .pixels()
        .map(|p| {
            u32::from(p[0])
                | (u32::from(p[1]) << 8)
                | (u32::from(p[2]) << 16)
                | (u32::from(p[3]) << 24)
        })
        .collect();

    let ctx = init_gpu()?;
    let processed = run_compute(&ctx, ENCODE_SHADER, "encode", &pixels)?;

    let mut rows = Vec::with_capacity(height as usize);
    for y in 0..height {
        let mut row = Vec::with_capacity(width as usize);
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let packed = processed[idx];
            let r = (packed & 0xFF) as u8;
            let g = ((packed >> 8) & 0xFF) as u8;
            let b = ((packed >> 16) & 0xFF) as u8;
            let a = ((packed >> 24) & 0xFF) as u8;
            row.push(Pixel {
                x,
                y,
                hex: schema::rgba_to_hex(r, g, b, a),
            });
        }
        rows.push(row);
    }

    Ok(rows)
}

pub fn decode_buffer(doc: &ImageJson) -> anyhow::Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let width = doc.width;
    let height = doc.height;
    let count = (width as usize) * (height as usize);

    let mut packed = Vec::with_capacity(count);
    for row in &doc.rows {
        for pixel in row {
            let rgba = schema::hex_to_rgba(&pixel.hex)
                .ok_or_else(|| anyhow::anyhow!("invalid hex: {}", pixel.hex))?;
            packed.push(
                u32::from(rgba[0])
                    | (u32::from(rgba[1]) << 8)
                    | (u32::from(rgba[2]) << 16)
                    | (u32::from(rgba[3]) << 24),
            );
        }
    }

    let ctx = init_gpu()?;
    let processed = run_compute(&ctx, DECODE_SHADER, "decode", &packed)?;

    let mut buffer = vec![0u8; count * 4];
    for (idx, value) in processed.iter().enumerate() {
        let offset = idx * 4;
        buffer[offset] = (value & 0xFF) as u8;
        buffer[offset + 1] = ((value >> 8) & 0xFF) as u8;
        buffer[offset + 2] = ((value >> 16) & 0xFF) as u8;
        buffer[offset + 3] = ((value >> 24) & 0xFF) as u8;
    }

    ImageBuffer::from_raw(width, height, buffer)
        .ok_or_else(|| anyhow::anyhow!("failed to build image buffer from GPU output"))
}
