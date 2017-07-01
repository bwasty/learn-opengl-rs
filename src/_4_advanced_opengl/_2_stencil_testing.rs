#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate glfw;
use self::glfw::Context;

extern crate gl;
use self::gl::types::*;

use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::ffi::CStr;

use common::{process_events, processInput, loadTexture};
use shader::Shader;
use camera::Camera;

use cgmath::{Matrix4, vec3, Deg, perspective, Point3};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_4_2() {
    let mut camera = Camera {
        Position: Point3::new(0.0, 0.0, 3.0),
        ..Camera::default()
    };

    let mut firstMouse = true;
    let mut lastX: f32 = SCR_WIDTH as f32 / 2.0;
    let mut lastY: f32 = SCR_HEIGHT as f32 / 2.0;

    // timing
    let mut deltaTime: f32; // time between current frame and last frame
    let mut lastFrame: f32 = 0.0;

    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);

    // tell GLFW to capture our mouse
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader, shaderSingleColor, cubeVBO, cubeVAO, planeVBO, planeVAO, cubeTexture, floorTexture) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::Enable(gl::STENCIL_TEST);
        gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);
        gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);

        // build and compile our shader program
        // ------------------------------------
        let shader = Shader::new(
            "src/_4_advanced_opengl/shaders/2.stencil_testing.vs",
            "src/_4_advanced_opengl/shaders/2.stencil_testing.fs");
        let shaderSingleColor = Shader::new(
            "src/_4_advanced_opengl/shaders/2.stencil_testing.vs",
            "src/_4_advanced_opengl/shaders/2.stencil_single_color.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let cubeVertices: [f32; 180] = [
             // positions       // texture Coords
             -0.5, -0.5, -0.5,  0.0, 0.0,
              0.5, -0.5, -0.5,  1.0, 0.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
             -0.5,  0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 0.0,

             -0.5, -0.5,  0.5,  0.0, 0.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 1.0,
              0.5,  0.5,  0.5,  1.0, 1.0,
             -0.5,  0.5,  0.5,  0.0, 1.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,

             -0.5,  0.5,  0.5,  1.0, 0.0,
             -0.5,  0.5, -0.5,  1.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,
             -0.5,  0.5,  0.5,  1.0, 0.0,

              0.5,  0.5,  0.5,  1.0, 0.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5,  0.5,  0.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 0.0,

             -0.5, -0.5, -0.5,  0.0, 1.0,
              0.5, -0.5, -0.5,  1.0, 1.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
              0.5, -0.5,  0.5,  1.0, 0.0,
             -0.5, -0.5,  0.5,  0.0, 0.0,
             -0.5, -0.5, -0.5,  0.0, 1.0,

             -0.5,  0.5, -0.5,  0.0, 1.0,
              0.5,  0.5, -0.5,  1.0, 1.0,
              0.5,  0.5,  0.5,  1.0, 0.0,
              0.5,  0.5,  0.5,  1.0, 0.0,
             -0.5,  0.5,  0.5,  0.0, 0.0,
             -0.5,  0.5, -0.5,  0.0, 1.0
        ];
        let planeVertices: [f32; 30] = [
            // positions       // texture Coords (note we set these higher than 1 (together with GL_REPEAT as texture wrapping mode). this will cause the floor texture to repeat)
             5.0, -0.5,  5.0,  2.0, 0.0,
            -5.0, -0.5,  5.0,  0.0, 0.0,
            -5.0, -0.5, -5.0,  0.0, 2.0,

             5.0, -0.5,  5.0,  2.0, 0.0,
            -5.0, -0.5, -5.0,  0.0, 2.0,
             5.0, -0.5, -5.0,  2.0, 2.0
        ];
        // cube VAO
        let (mut cubeVAO, mut cubeVBO) = (0, 0);
        gl::GenVertexArrays(1, &mut cubeVAO);
        gl::GenBuffers(1, &mut cubeVBO);
        gl::BindVertexArray(cubeVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, cubeVBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (cubeVertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &cubeVertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::BindVertexArray(0);
        // plane VAO
        let (mut planeVAO, mut planeVBO) = (0, 0);
        gl::GenVertexArrays(1, &mut planeVAO);
        gl::GenBuffers(1, &mut planeVBO);
        gl::BindVertexArray(planeVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, planeVBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (planeVertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &planeVertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::BindVertexArray(0);

        // load textures
        // -------------
        let cubeTexture = loadTexture("resources/textures/marble.jpg");
        let floorTexture = loadTexture("resources/textures/metal.png");

        // shader configuration
        // --------------------
        shader.useProgram();
        shader.setInt(c_str!("texture1"), 0);

        (shader, shaderSingleColor, cubeVBO, cubeVAO, planeVBO, planeVAO, cubeTexture, floorTexture)
    };

    // render loop
    // -----------
    while !window.should_close() {
        // per-frame time logic
        // --------------------
        let currentFrame = glfw.get_time() as f32;
        deltaTime = currentFrame - lastFrame;
        lastFrame = currentFrame;

        // events
        // -----
        process_events(&events, &mut firstMouse, &mut lastX, &mut lastY, &mut camera);

        // input
        // -----
        processInput(&mut window, deltaTime, &mut camera);

        // render
        // ------
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(
                gl::COLOR_BUFFER_BIT |
                gl::DEPTH_BUFFER_BIT |
                gl::STENCIL_BUFFER_BIT); // don't forget to clear the stencil buffer!

            // set uniforms
            shaderSingleColor.useProgram();
            let mut model: Matrix4<f32>;
            let view = camera.GetViewMatrix();
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
            shaderSingleColor.setMat4(c_str!("view"), &view);
            shaderSingleColor.setMat4(c_str!("projection"), &projection);

            shader.useProgram();
            shader.setMat4(c_str!("view"), &view);
            shader.setMat4(c_str!("projection"), &projection);

            // draw floor as normal, but don't write the floor to the stencil buffer, we only care about the containers. We set its mask to 0x00 to not write to the stencil buffer.
            gl::StencilMask(0x00);
            // floor
            gl::BindVertexArray(planeVAO);
            gl::BindBuffer(gl::TEXTURE_2D, floorTexture);
            shader.setMat4(c_str!("model"), &Matrix4::identity());
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
            // 1st. render pass, draw objects as normal, writing to the stencil buffer
            // --------------------------------------------------------------------
            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
            gl::StencilMask(0xFF);
            // cubes
            gl::BindVertexArray(cubeVAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, cubeTexture);
            model = Matrix4::from_translation(vec3(-1.0, 0.0, -1.0));
            shader.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            model = Matrix4::from_translation(vec3(2.0, 0.0, 0.0));
            shader.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // 2nd. render pass: now draw slightly scaled versions of the objects, this time disabling stencil writing.
            // Because the stencil buffer is now filled with several 1s. The parts of the buffer that are 1 are not drawn, thus only drawing
            // the objects' size differences, making it look like borders.
            // -----------------------------------------------------------------------------------------------------------------------------
            gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);
            gl::StencilMask(0x00);
            gl::Disable(gl::DEPTH_TEST);
            shaderSingleColor.useProgram();
            let scale = 1.1;
            // cubes
            gl::BindVertexArray(cubeVAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, cubeTexture);
            model = Matrix4::from_translation(vec3(-1.0, 0.0, -1.0));
            model = model * Matrix4::from_scale(scale);
            shader.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            model = Matrix4::from_translation(vec3(2.0, 0.0, 0.0));
            model = model * Matrix4::from_scale(scale);
            shader.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);
            gl::StencilMask(0xFF);
            gl::Enable(gl::DEPTH_TEST);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    unsafe {
        gl::DeleteVertexArrays(1, &cubeVAO);
        gl::DeleteVertexArrays(1, &planeVAO);
        gl::DeleteBuffers(1, &cubeVBO);
        gl::DeleteBuffers(1, &planeVBO);
    }
}
