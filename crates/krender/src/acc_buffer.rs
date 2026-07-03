use std::sync::Arc;

use glow::HasContext;
use kmath::Vec2i;

pub struct AccBuffer {
    gl: Arc<glow::Context>,

    write_fbo: glow::NativeFramebuffer,
    read_fbo: glow::NativeFramebuffer,

    write_tex: glow::NativeTexture,
    read_tex: glow::NativeTexture,

    size: Vec2i,
}

impl AccBuffer {
    pub fn new(gl: Arc<glow::Context>, size: Vec2i) -> Result<Self, String> {
        unsafe {
            let fbo = [gl.create_framebuffer()?, gl.create_framebuffer()?];

            let tex = [gl.create_texture()?, gl.create_texture()?];

            for i in 0..2 {
                gl.bind_texture(glow::TEXTURE_2D, Some(tex[i]));

                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA32F as i32,
                    size.x(),
                    size.y(),
                    0,
                    glow::RGBA,
                    glow::FLOAT,
                    glow::PixelUnpackData::Slice(None),
                );

                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::NEAREST as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::NEAREST as i32,
                );

                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_S,
                    glow::CLAMP_TO_EDGE as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_T,
                    glow::CLAMP_TO_EDGE as i32,
                );

                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo[i]));

                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::TEXTURE_2D,
                    Some(tex[i]),
                    0,
                );

                if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                    return Err(format!("Framebuffer {} is incomplete", i));
                }
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            Ok(Self {
                gl,
                write_fbo: fbo[0],
                read_fbo: fbo[1],
                write_tex: tex[0],
                read_tex: tex[1],
                size,
            })
        }
    }

    pub fn resize(&mut self, new_size: Vec2i) -> Result<(), String> {
        unsafe {
            for i in 0..2 {
                self.gl.bind_texture(
                    glow::TEXTURE_2D,
                    Some(if i == 0 {
                        self.write_tex
                    } else {
                        self.read_tex
                    }),
                );

                self.gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA32F as i32,
                    new_size.x(),
                    new_size.y(),
                    0,
                    glow::RGBA,
                    glow::FLOAT,
                    glow::PixelUnpackData::Slice(None),
                );
            }
        }

        self.size = new_size;

        Ok(())
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.read_tex, &mut self.write_tex);
        std::mem::swap(&mut self.read_fbo, &mut self.write_fbo);
    }

    pub fn bind_write_tex(&self) {
        unsafe {
            self.gl.bind_image_texture(
                0,
                Some(self.write_tex),
                0,
                false,
                0,
                glow::WRITE_ONLY,
                glow::RGBA32F,
            );
        }
    }

    pub fn bind_read_tex(&self, unit: u32) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 + unit);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.read_tex));
        }
    }

    pub fn blit(&self) {
        unsafe {
            self.gl
                .bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.read_fbo));
            self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);

            self.gl.blit_framebuffer(
                0,
                0,
                self.size.x(),
                self.size.y(),
                0,
                0,
                self.size.x(),
                self.size.y(),
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );

        self.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
        }
    }
}

impl Drop for AccBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffer(self.write_fbo);
            self.gl.delete_framebuffer(self.read_fbo);

            self.gl.delete_texture(self.write_tex);
            self.gl.delete_texture(self.read_tex);
        }
    }
}
