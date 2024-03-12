use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::render::WindowCanvas;

use crate::error::ApplicationResult;

const WIDTH: u32 = 8;
const HEIGHT: u32 = 16;

pub struct Font<'a> {
    width: u32,
    height: u32,
    texture: Texture<'a>,
}

impl<'a> Font<'a> {
    pub fn create<'b, T>(
        width: u32,
        height: u32,
        texture_creator: &'b TextureCreator<T>,
    ) -> ApplicationResult<Font<'b>> {
        let texture = texture_creator.load_texture_bytes(include_bytes!("font.png"))?;

        Ok(Font {
            width,
            height,
            texture,
        })
    }

    pub fn write(
        &self,
        canvas: &mut WindowCanvas,
        x: i32,
        y: i32,
        text: &str,
    ) -> ApplicationResult<()> {
        let mut cursor_x = x;
        let mut cursor_y = y;

        for ch in text.chars() {
            match ch {
                ch if ch == '\n' => {
                    cursor_x = x;
                    cursor_y += self.height as i32;
                }
                ch if ch < ' ' => {}
                ch if ch < '\x7f' => {
                    let offset_x = WIDTH * ((ch as u32) % 16);
                    let offset_y = HEIGHT * ((ch as u32) / 16);

                    canvas.copy(
                        &self.texture,
                        Rect::new(offset_x as i32, offset_y as i32, WIDTH, HEIGHT),
                        Rect::new(cursor_x, cursor_y, self.width, self.height),
                    )?;

                    cursor_x += self.width as i32;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
