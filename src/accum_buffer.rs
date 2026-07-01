#![allow(unsafe_op_in_unsafe_fn)]

pub struct AccumBuffer {
    pub fbo: [u32; 2],
    pub tex: [u32; 2],
    pub width: i32,
    pub height: i32,
}

impl AccumBuffer {
    pub unsafe fn new(width: i32, height: i32) -> Self {
        let mut tex = [0u32; 2];
        let mut fbo = [0u32; 2];
        gl::GenTextures(2, tex.as_mut_ptr());
        gl::GenFramebuffers(2, fbo.as_mut_ptr());

        for i in 0..2 {
            gl::BindTexture(gl::TEXTURE_2D, tex[i]);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::FLOAT,
                std::ptr::null(),
            );
            // NEAREST: we never want filtering to blend across pixels here.
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo[i]);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                tex[i],
                0,
            );

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Accum framebuffer {i} incomplete: 0x{status:x}");
            }
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        Self {
            fbo,
            tex,
            width,
            height,
        }
    }

    pub unsafe fn destroy(&self) {
        gl::DeleteFramebuffers(2, self.fbo.as_ptr());
        gl::DeleteTextures(2, self.tex.as_ptr());
    }
}