use image::{imageops::FilterType, DynamicImage, FlatSamples};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
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

    #[error("Unable to convert image to RGB8")]
    ImageConversionError,

    #[error("Cannot find primary monitor")]
    NoPrimaryMonitor,
}

// Define type for main return result to auto convert error
type Result<T> = std::result::Result<T, RviError>;

fn main() -> Result<()> {
    let config: Config = Config::parse();

    let stream_image: DynamicImage = image::io::Reader::open(&config.file_name)?
        .with_guessed_format()?
        .decode()?;

    let event_loop: EventLoop<()> = EventLoop::new();
    let primary_monitor: MonitorHandle = event_loop
        .primary_monitor()
        .ok_or(RviError::NoPrimaryMonitor)?;

    let screen_size: PhysicalSize<u32> = primary_monitor.size();

    // Get an array of scale float sizes for scaling
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
    // Assign highest scale factor too scale so that it always scales
    // inside the window's bounds
    let scale: f32 = scale[1];

    let window_inner_size: PhysicalSize<u32> = PhysicalSize::new(
        (stream_image.width() as f32 / scale).ceil() as u32,
        (stream_image.height() as f32 / scale).ceil() as u32,
    );

    let window: Window = WindowBuilder::new()
        .with_title("RIV")
        .with_inner_size(window_inner_size)
        .build(&event_loop)?;

    let surface = SurfaceTexture::new(window_inner_size.width, window_inner_size.height, &window);
    let mut pixels = Pixels::new(200, 200, surface)?;

    redraw_surface(&mut pixels, &window_inner_size, &stream_image)?;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            winit::event::Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    winit::event::WindowEvent::Resized(size) => {
                        redraw_surface(&mut pixels, &size, &stream_image).unwrap();
                    }
                    winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => match virtual_keycode.unwrap() {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        VirtualKeyCode::R => {
                            redraw_surface(&mut pixels, &window.inner_size(), &stream_image)
                                .unwrap()
                        }
                        _ => {}
                    },

                    winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        redraw_surface(&mut pixels, new_inner_size, &stream_image).unwrap();
                    }

                    _ => {}
                }
            }
            winit::event::Event::RedrawRequested(_) => {
                let _ = pixels.render();
            }

            _ => {}
        }
    })
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
    let image: DynamicImage = stream_image.resize(size.width, size.height, FilterType::Triangle);

    // Use new build image to resize the pixels buffer
    pixels.resize_buffer(image.width(), image.height());
    pixels.resize_surface(size.width, size.height);

    let image_bytes: FlatSamples<&[u8]> = image
        .as_rgb8()
        .ok_or(RviError::ImageConversionError)?
        .as_flat_samples();
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

    pixels.render()?;

    Ok(())
}
