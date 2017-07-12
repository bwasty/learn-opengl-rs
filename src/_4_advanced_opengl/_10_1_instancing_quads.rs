#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::ptr;
use std::mem;
use std::os::raw::c_void;

extern crate glfw;
use self::glfw::Context;

extern crate gl;
use self::gl::types::*;

use cgmath::{Vector2};

use num::range_step;

use shader::Shader;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_4_10_1() {
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, _events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_framebuffer_size_polling(true);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader, quadVAO, quadVBO) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile shaders
        // -------------------------
        let shader = Shader::new(
            "src/_4_advanced_opengl/shaders/10.1.instancing.vs",
            "src/_4_advanced_opengl/shaders/10.1.instancing.fs",
        );

        // generate a list of 100 quad locations/translation-vectors
        // ---------------------------------------------------------
        let mut translations = vec![];
        let offset = 0.1;
        for y in range_step(-10, 10, 2) {
            for x in range_step(-10, 10, 2) {
                translations.push(
                    Vector2 {
                        x: x as i32 as f32 / 10.0 + offset,
                        y: y as i32 as f32 / 10.0 + offset
                    }
                )
            }
        }

        let mut instanceVBO = 0;
        gl::GenBuffers(1, &mut instanceVBO);
        gl::BindBuffer(gl::ARRAY_BUFFER, instanceVBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of::<Vector2<f32>>() as isize * 100 ,
            &translations[0] as *const Vector2<f32> as *const c_void,
            gl::STATIC_DRAW);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let quadVertices: [f32; 30] = [
            // positions   // colors
            -0.05,  0.05,  1.0, 0.0, 0.0,
             0.05, -0.05,  0.0, 1.0, 0.0,
            -0.05, -0.05,  0.0, 0.0, 1.0,

            -0.05,  0.05,  1.0, 0.0, 0.0,
             0.05, -0.05,  0.0, 1.0, 0.0,
             0.05,  0.05,  0.0, 1.0, 1.0
        ];
        let (mut quadVAO, mut quadVBO) = (0, 0);
        gl::GenVertexArrays(1, &mut quadVAO);
        gl::GenBuffers(1, &mut quadVBO);
        gl::BindVertexArray(quadVBO);
        gl::BindBuffer(gl::ARRAY_BUFFER, quadVBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (quadVertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &quadVertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const c_void);
        // also set instance data
        gl::EnableVertexAttribArray(2);
        gl::BindBuffer(gl::ARRAY_BUFFER, instanceVBO); // this attribute comes from a different vertex buffer
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, (2 * mem::size_of::<GLfloat>()) as i32, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::VertexAttribDivisor(2, 1); // tell OpenGL this is an instanced vertex attribute.

        (shader, quadVAO, quadVBO)
    };

    // render loop
    // -----------
    while !window.should_close() {
        // render
        // ------
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // draw 100 instanced quads
            shader.useProgram();
            gl::BindVertexArray(quadVAO);
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, 100); // 100 triangles of 6 vertices each
            gl::BindVertexArray(0);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

        // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    unsafe {
        gl::DeleteVertexArrays(1, &quadVAO);
        gl::DeleteBuffers(1, &quadVBO);
    }
}
