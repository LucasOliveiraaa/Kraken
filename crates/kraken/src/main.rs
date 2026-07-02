mod accum_buffer;
mod camera;
mod config_buffer;
mod gl_util;
mod profile_buffer;
mod scene;
mod structure;

use std::fs;

use glfw::{Action, Context, Key, WindowMode};

use accum_buffer::AccumBuffer;
use camera::{Camera, halton_jitter};
use config_buffer::ConfigBuffer;
use gl_util::{compile_shader, link_program, uniform_loc};
use kmath::{Mat4, Vec2, Vec3};
use profile_buffer::ProfileBuffer;
use scene::{SceneBuffers, build_mock_scene};

fn main() {
    tracy_client::Client::start();

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // Request a core profile so gl_VertexID works without a bound VBO.
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(4));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::RefreshRate(None));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let resolution = Vec2::new(1366.0, 768.0);

    let (mut window, events) = glfw.with_primary_monitor(|fw, _| {
        fw.create_window(
            resolution.x as u32,
            resolution.y as u32,
            "Kraken GL COMP",
            WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window")
    });

    window.make_current();
    window.set_key_polling(true);
    gl::load_with(|s| window.get_proc_address(s) as *const _);
    glfw.set_swap_interval(glfw::SwapInterval::None);

    // ---------- shaders ----------
    let comp_src =
        fs::read_to_string("assets/shaders/raytracer.comp").expect("Failed to read compute shader");

    let comp_shader = compile_shader(&comp_src, gl::COMPUTE_SHADER);
    let program = link_program(comp_shader);
    unsafe {
        gl::DeleteShader(comp_shader);
    }

    // Dummy VAO — required by the core profile even though the fullscreen
    // triangle's vertices are generated in the vertex shader from
    // gl_VertexID.
    let mut vao = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }

    // ---------- scene ----------
    let scene_data = build_mock_scene();
    let scene_buffers = unsafe { SceneBuffers::upload(&scene_data) };

    let aspect = resolution.x / resolution.y;
    let inv_proj = Mat4::from_perspective(1.0472, aspect, 0.1, 100.0).inverse();

    // ---------- camera ----------
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 7.5));
    let mut prev_cam = (camera.pos, camera.yaw, camera.pitch);

    // ---------- accumulation / profiling / config ----------
    let accum = unsafe { AccumBuffer::new(resolution.x as i32, resolution.y as i32) };
    let mut profile_buffer = unsafe { ProfileBuffer::new(scene::bindings::PROFILE) };
    let mut config = unsafe { ConfigBuffer::new(scene::bindings::CONFIG) };

    let mut frame_index: u32 = 0;
    let mut write_idx = 0usize;

    let mut query = 0;
    unsafe {
        gl::GenQueries(1, &mut query);
    }

    let mut last_time = glfw.get_time() as f32;
    let mut time_accum = 0.0f64;
    let mut fps = 0;

    // ---------- render loop ----------
    while !window.should_close() {
        tracy_client::frame_mark();
        let _span = tracy_client::span!("Frame");

        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                // Tab cycles through the available debug view modes.
                glfw::WindowEvent::Key(Key::Tab, _, Action::Press, _) => {
                    config.cycle_view_mode();
                }
                _ => {}
            }
        }

        let dt = glfw.get_time() as f32 - last_time;
        last_time += dt;

        time_accum += dt as f64;
        if time_accum >= 1.0 {
            window.set_title(&format!(
                "Kraken GL COMP - FPS: {} - View: {}",
                fps,
                config.view_mode().label()
            ));
            time_accum -= 1.0;
            fps = 0;
        }
        fps += 1;

        {
            let _span = tracy_client::span!("Camera update");
            camera.handle_input(&window, dt);
        }

        let cur_cam = (camera.pos, camera.yaw, camera.pitch);
        let camera_moved = cur_cam != prev_cam;
        // Any camera movement or a view-mode change invalidates the
        // temporal accumulation history.
        if camera_moved || config.dirty {
            frame_index = 0;
        }
        prev_cam = cur_cam;

        let inv_view = camera.inv_view();
        let read_idx = 1 - write_idx;
        let jitter = halton_jitter(frame_index);

        unsafe {
            gl::BeginQuery(gl::TIME_ELAPSED, query);

            let _span = tracy_client::span!("Render pass");

            gl::BindImageTexture(
                0,
                accum.tex[write_idx],
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA32F,
            );

            gl::Viewport(0, 0, accum.width, accum.height);
            gl::UseProgram(program);

            {
                let _span = tracy_client::span!("Buffer Binding");

                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, accum.tex[read_idx]);
                gl::Uniform1i(uniform_loc(program, "historyTex"), 0);
                gl::Uniform1ui(uniform_loc(program, "frameIndex"), frame_index);
                gl::Uniform2f(uniform_loc(program, "jitter"), jitter.x, jitter.y);

                gl::Uniform1ui(
                    uniform_loc(program, "volumeCount"),
                    scene_data.volumes.len() as u32,
                );

                gl::UniformMatrix4fv(
                    uniform_loc(program, "invView"),
                    1,
                    gl::FALSE,
                    inv_view.as_ptr(),
                );
                gl::UniformMatrix4fv(
                    uniform_loc(program, "invProj"),
                    1,
                    gl::FALSE,
                    inv_proj.as_ptr(),
                );
                gl::Uniform3f(
                    uniform_loc(program, "cameraPos"),
                    camera.pos.x,
                    camera.pos.y,
                    camera.pos.z,
                );
                gl::Uniform2f(
                    uniform_loc(program, "resolution"),
                    resolution.x,
                    resolution.y,
                );

                profile_buffer.begin_frame();
                config.upload_if_dirty();
            }

            gl::DispatchCompute(
                (accum.width as u32 + 15) / 16,
                (accum.height as u32 + 15) / 16,
                1,
            );
            gl::MemoryBarrier(gl::ALL_BARRIER_BITS);

            profile_buffer.end_frame();

            let _span = tracy_client::span!("Present");
            // Present: copy the freshly-written accum texture to the screen.
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, accum.fbo[write_idx]);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::BlitFramebuffer(
                0,
                0,
                accum.width,
                accum.height,
                0,
                0,
                accum.width,
                accum.height,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );

            gl::EndQuery(gl::TIME_ELAPSED);
        }

        write_idx = read_idx;
        frame_index += 1;
        window.swap_buffers();

        unsafe {
            profile_buffer.poll_and_report();

            let mut available = 0;
            gl::GetQueryObjectiv(query, gl::QUERY_RESULT_AVAILABLE, &mut available);
            if available != 0 {
                let mut ns: u64 = 0;
                gl::GetQueryObjectui64v(query, gl::QUERY_RESULT, &mut ns);
                let ms = ns as f64 / 1_000_000.0;
                tracy_client::plot!("GPU Time (ms)", ms);
            }
        }
    }

    // ---------- cleanup ----------
    unsafe {
        scene_buffers.destroy();
        accum.destroy();
        profile_buffer.destroy();
        config.destroy();
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteProgram(program);
    }
}