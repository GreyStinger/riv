use super::errors::Result;
use image::{DynamicImage, FlatSamples, imageops::FilterType};
use pixels::Pixels;
use winit::dpi::PhysicalSize;

pub fn redraw_surface(
    pixels: &mut Pixels,
    size: &PhysicalSize<u32>,
    stream_image: &DynamicImage,
) -> Result<()> {
    if cfg!(debug_assertions) {
        println!("Attempting resize on image");
    }
    let image: DynamicImage = resize_image(stream_image, size.width, size.height);

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

pub fn resize_image(
    image: &DynamicImage, 
    width: u32, 
    height: u32
) -> DynamicImage {
    image.resize(width, height, FilterType::Nearest)
}
