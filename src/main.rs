use std::option_env;

use image::{imageops::FilterType, DynamicImage, FlatSamples};
use pixels::wgpu::RequestAdapterOptions;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    monitor::MonitorHandle,
    window::{Window, WindowBuilder},
};

use clap::Parser;
use thiserror::Error;

const SCREEN_PERCENT: u32 = 90;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Config {
    /// Name of image to open
    file_name: String,

    /// Wether to scale the image up
    #[clap(short, long, takes_value = false)]
    up_scale: bool,

    /// Whether to force integrated gpu
    #[clap(short, long, takes_value = false)]
    low_performance_mode: bool,
}

// Custom error type to auto handle any errors in main thread
#[derive(Debug, Error)]
enum RviError {
    #[error("Unable to create window")]
    WindowError(#[from] winit::error::OsError),

    #[error("An error occurred while processing the image")]
    ImageError(#[from] image::ImageError),

    #[error("And error occurred while loading the image")]
    IoError(#[from] std::io::Error),

    #[error("Unable to create new pixels instance")]
    PixelsError(#[from] pixels::Error),

    #[error("Cannot find primary monitor")]
    NoPrimaryMonitor,
}

#[derive(Debug)]
enum CustomWindowEvent {
    Rebuild,
}

// Define type for main return result to auto convert error
type Result<T> = std::result::Result<T, RviError>;

fn main() -> Result<()> {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_BACKTRACE", "full");
    };
    let config: Config = Config::parse();

    if cfg!(debug_assertions) {
        println!("Fetching and decoding stream image");
    }
    let stream_image: DynamicImage = image::io::Reader::open(&config.file_name)?
        .with_guessed_format()?
        .decode()?;

    if cfg!(debug_assertions) {
        println!("Creating event loop");
    }
    let event_loop: EventLoop<CustomWindowEvent> =
        winit::event_loop::EventLoopBuilder::<CustomWindowEvent>::with_user_event().build();
    let event_loop_proxy: EventLoopProxy<CustomWindowEvent> = event_loop.create_proxy();

    let screen_size: PhysicalSize<u32> = match get_screen_size(&event_loop) {
        Ok(screen_size) => screen_size,
        Err(_) => {
            let ss: (u32, u32) = match option_env!("SCREEN_SIZE") as Option<&str> {
                Some(ss) => {
                    let ss: Vec<&str> = ss.splitn(2, ",").collect();
                    (ss[0].parse().unwrap(), ss[1].parse().unwrap())
                }
                None => (640, 480),
            };
            PhysicalSize::new(ss.0, ss.1)
        }
    };
    if cfg!(debug_assertions) { dbg!(screen_size); }

    let mut scale: [f32; 2] = [
        calc_scale_factor(
            &(screen_size.width * SCREEN_PERCENT / 100),
            &stream_image.width(),
            Some(config.up_scale),
        ),
        calc_scale_factor(
            &(screen_size.height * SCREEN_PERCENT / 100),
            &stream_image.height(),
            Some(config.up_scale),
        ),
    ];

    float_ord::sort(&mut scale);

    let scale: f32 = scale[1];

    let window_inner_size: PhysicalSize<u32> = PhysicalSize::new(
        (stream_image.width() as f32 / scale).ceil() as u32,
        (stream_image.height() as f32 / scale).ceil() as u32,
    );

    if cfg!(debug_assertions) {
        println!("Creating a new window");
    }
    let window: Window = WindowBuilder::new()
        .with_title("RIV")
        .with_inner_size(window_inner_size)
        .with_position(PhysicalPosition::new(20, 20))
        .build(&event_loop)?;
    let surface: SurfaceTexture<Window> =
        SurfaceTexture::new(window_inner_size.width, window_inner_size.height, &window);

    if cfg!(debug_assertions) {
        println!("Building initial pixels with low performance mode as:");
        dbg!(config.low_performance_mode);
        // Enumerate adapters
        let instance = pixels::wgpu::Instance::new(pixels::wgpu::Backends::all());
        for adapter in instance.enumerate_adapters(pixels::wgpu::Backends::all()) {
            dbg!(adapter);
        }
    }
    let mut pixels: Pixels = PixelsBuilder::new(200, 200, surface)
        .device_descriptor(pixels::wgpu::DeviceDescriptor {
            features: pixels::wgpu::Features::empty(),
            limits: pixels::wgpu::Limits::default(),
            label: None,
        })
        .request_adapter_options(RequestAdapterOptions {
            power_preference: if config.low_performance_mode {
                pixels::wgpu::PowerPreference::default()
            } else {
                pixels::wgpu::PowerPreference::HighPerformance
            },
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .wgpu_backend(pixels::wgpu::Backends::all())
        .enable_vsync(false)
        .build()?;

    redraw_surface(&mut pixels, &window_inner_size, &stream_image)?;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            winit::event::Event::UserEvent(event) => match event {
                CustomWindowEvent::Rebuild => {
                    redraw_surface(&mut pixels, &window.inner_size(), &stream_image).unwrap();
                }
            },

            winit::event::Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => match virtual_keycode {
                        Some(kc) => match kc {
                            VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                            VirtualKeyCode::Q => *control_flow = ControlFlow::Exit,
                            VirtualKeyCode::R => {
                                redraw_surface(&mut pixels, &window.inner_size(), &stream_image)
                                    .unwrap()
                            }
                            _ => {}
                        },
                        None => {}
                    },

                    _ => {}
                }
            }

            winit::event::Event::RedrawRequested(_) => {
                // Emit custom rebuild event on screen redraw
                event_loop_proxy
                    .send_event(CustomWindowEvent::Rebuild)
                    .unwrap();
            }

            _ => {}
        }
    })
}

fn get_screen_size(event_loop: &EventLoop<CustomWindowEvent>) -> Result<PhysicalSize<u32>> {
    let primary_monitor: MonitorHandle = event_loop
        .primary_monitor()
        .ok_or(RviError::NoPrimaryMonitor)?;

    Ok(primary_monitor.size())
}

fn calc_scale_factor(max_size: &u32, current_size: &u32, up_scale: Option<bool>) -> f32 {
    if max_size >= current_size && !up_scale.unwrap_or(false) {
        return 1 as f32;
    }
    *current_size as f32 / *max_size as f32
}

// Redraw and resize the surface and buffer inside the window
//
// @return Result((), RviError)
fn redraw_surface(
    pixels: &mut Pixels,
    size: &PhysicalSize<u32>,
    stream_image: &DynamicImage,
) -> Result<()> {
    if cfg!(debug_assertions) {
        println!("Attempting resize on image");
    }
    let image: DynamicImage = stream_image.resize(size.width, size.height, FilterType::Triangle);

    // Use new build image to resize the pixels buffer
    pixels.resize_buffer(image.width(), image.height());
    pixels.resize_surface(size.width, size.height);

    if cfg!(debug_assertions) {
        println!("Converting image to rgb8");
    }
    let rgb8_image = image.into_rgb8();
    let image_bytes: FlatSamples<&[u8]> = rgb8_image.as_flat_samples();
    let image_bytes: &[u8] = image_bytes.as_slice();

    image_bytes
        .chunks_exact(3)
        .zip(pixels.get_frame().chunks_exact_mut(4))
        .for_each(|(image_pixel, pixel)| {
            pixel[0] = image_pixel[0];
            pixel[1] = image_pixel[1];
            pixel[2] = image_pixel[2];
            pixel[3] = 0xff;
        });

    if cfg!(debug_assertions) {
        println!("Rendering pixels");
    }
    pixels.render()?;

    Ok(())
}
