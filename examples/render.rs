//! Example program for testing rendering with skribo.

use std::fs::File;
use std::io::Write;

use euclid::{Point2D, Size2D};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::loaders::default::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

use skribo::{make_layout, Layout, TextStyle};

struct SimpleSurface {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

fn composite(a: u8, b: u8) -> u8 {
    let y = ((255 - a) as u16) * ((255 - b) as u16);
    let y = (y + (y >> 8) + 0x80) >> 8; // fast approx to round(y / 255)
    255 - (y as u8)
}

// A simple drawing surface, just because it's easier to implement such things
// directly than pull in dependencies for it.
impl SimpleSurface {
    fn new(width: usize, height: usize) -> SimpleSurface {
        let pixels = vec![0; width * height];
        SimpleSurface {
            width,
            height,
            pixels,
        }
    }

    fn paint_from_canvas(&mut self, canvas: &Canvas, x: i32, y: i32) {
        let (cw, ch) = (canvas.size.width as i32, canvas.size.height as i32);
        let (w, h) = (self.width as i32, self.height as i32);
        let xmin = 0.max(-x);
        let xmax = cw.min(w - x);
        let ymin = 0.max(-y);
        let ymax = ch.min(h - y);
        for yy in ymin..(ymax.max(ymin)) {
            for xx in xmin..(xmax.max(xmin)) {
                let pix = canvas.pixels[(cw * yy + xx) as usize];
                let dst_ix = ((y + yy) * w + x + xx) as usize;
                self.pixels[dst_ix] = composite(self.pixels[dst_ix], pix);
            }
        }
    }

    fn write_pgm(&self, filename: &str) -> Result<(), std::io::Error> {
        let mut f = File::create(filename)?;
        write!(f, "P5\n{} {}\n255\n", self.width, self.height)?;
        f.write(&self.pixels)?;
        Ok(())
    }

    fn paint_layout(&mut self, font: &Font, layout: &Layout, x: i32, y: i32) {
        for glyph in &layout.glyphs {
            let glyph_id = glyph.glyph_id;
            let glyph_x = (glyph.offset.x as i32) + x;
            let glyph_y = (glyph.offset.y as i32) + y;
            let bounds = font
                .raster_bounds(
                    glyph_id,
                    layout.size,
                    &Point2D::zero(),
                    HintingOptions::None,
                    RasterizationOptions::GrayscaleAa,
                )
                .unwrap();
            println!(
                "glyph {}, bounds {:?}, {},{}",
                glyph_id, bounds, glyph_x, glyph_y
            );
            if !bounds.is_empty() {
                let mut canvas = Canvas::new(
                    &Size2D::new(bounds.size.width as u32, bounds.size.height as u32),
                    Format::A8,
                );
                font.rasterize_glyph(
                    &mut canvas,
                    glyph_id,
                    // TODO(font-kit): this is missing anamorphic and skew features
                    layout.size,
                    &Point2D::zero(), // TODO: include origin
                    HintingOptions::None,
                    RasterizationOptions::GrayscaleAa,
                )
                .unwrap();
                self.paint_from_canvas(&canvas, glyph_x, glyph_y);
            }
        }
    }
}

fn main() {
    println!("render test");
    let font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap()
        .load()
        .unwrap();
    let style = TextStyle { size: 32.0 };
    let glyph_id = font.glyph_for_char('O').unwrap();
    println!("glyph id = {}", glyph_id);
    println!(
        "glyph typo bounds: {:?}",
        font.typographic_bounds(glyph_id).unwrap()
    );
    println!(
        "glyph raster bounds: {:?}",
        font.raster_bounds(
            glyph_id,
            32.0,
            &Point2D::zero(),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa
        )
    );
    let mut canvas = Canvas::new(&Size2D::new(32, 32), Format::A8);
    font.rasterize_glyph(
        &mut canvas,
        glyph_id,
        // TODO(font-kit): this is missing anamorphic and skew features
        style.size,
        &Point2D::zero(),
        HintingOptions::None,
        RasterizationOptions::GrayscaleAa,
    )
    .unwrap();
    // TODO(font-kit): FreeType is top-aligned, CoreText is bottom-aligned, and FT seems to ignore origin
    font.rasterize_glyph(
        &mut canvas,
        glyph_id,
        style.size,
        &Point2D::new(16.0, 16.0),
        HintingOptions::None,
        RasterizationOptions::GrayscaleAa,
    )
    .unwrap();

    let layout = make_layout(&style, &font, "hello world");
    println!("{:?}", layout);
    let mut surface = SimpleSurface::new(200, 50);
    surface.paint_layout(&font, &layout, 0, 0);
    surface.write_pgm("out.pgm").unwrap();
}