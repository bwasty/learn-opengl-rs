#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::ffi::CStr;

extern crate glfw;
use self::glfw::Context;

extern crate gl;
use self::gl::types::*;

use cgmath::{Matrix4, vec3, Deg, perspective, Point3};
use cgmath::prelude::*;

use common::{process_events, processInput};
use shader::Shader;
use camera::Camera;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_4_8() {
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

    let (shaderRed, shaderGreen, shaderBlue, shaderYellow, cubeVBO, cubeVAO, uboMatrices) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile shaders
        // -------------------------
        let shaderRed = Shader::new("src/_4_advanced_opengl/shaders/8.advanced_glsl.vs", "src/_4_advanced_opengl/shaders/8.red.fs");
        let shaderGreen = Shader::new("src/_4_advanced_opengl/shaders/8.advanced_glsl.vs", "src/_4_advanced_opengl/shaders/8.green.fs");
        let shaderBlue = Shader::new("src/_4_advanced_opengl/shaders/8.advanced_glsl.vs", "src/_4_advanced_opengl/shaders/8.blue.fs");
        let shaderYellow = Shader::new("src/_4_advanced_opengl/shaders/8.advanced_glsl.vs", "src/_4_advanced_opengl/shaders/8.yellow.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let cubeVertices: [f32; 108] = [
            // positions
            -0.5, -0.5, -0.5,
             0.5, -0.5, -0.5,
             0.5,  0.5, -0.5,
             0.5,  0.5, -0.5,
            -0.5,  0.5, -0.5,
            -0.5, -0.5, -0.5,

            -0.5, -0.5,  0.5,
             0.5, -0.5,  0.5,
             0.5,  0.5,  0.5,
             0.5,  0.5,  0.5,
            -0.5,  0.5,  0.5,
            -0.5, -0.5,  0.5,

            -0.5,  0.5,  0.5,
            -0.5,  0.5, -0.5,
            -0.5, -0.5, -0.5,
            -0.5, -0.5, -0.5,
            -0.5, -0.5,  0.5,
            -0.5,  0.5,  0.5,

             0.5,  0.5,  0.5,
             0.5,  0.5, -0.5,
             0.5, -0.5, -0.5,
             0.5, -0.5, -0.5,
             0.5, -0.5,  0.5,
             0.5,  0.5,  0.5,

            -0.5, -0.5, -0.5,
             0.5, -0.5, -0.5,
             0.5, -0.5,  0.5,
             0.5, -0.5,  0.5,
            -0.5, -0.5,  0.5,
            -0.5, -0.5, -0.5,

            -0.5,  0.5, -0.5,
             0.5,  0.5, -0.5,
             0.5,  0.5,  0.5,
             0.5,  0.5,  0.5,
            -0.5,  0.5,  0.5,
            -0.5,  0.5, -0.5,
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
        let stride = 3 * mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());

        // configure a uniform buffer object
        // ---------------------------------
        // first. We get the relevant block indices
        let uniformBlockIndexRed = gl::GetUniformBlockIndex(shaderRed.ID, c_str!("Matrices").as_ptr());
        let uniformBlockIndexGreen = gl::GetUniformBlockIndex(shaderGreen.ID, c_str!("Matrices").as_ptr());
        let uniformBlockIndexBlue = gl::GetUniformBlockIndex(shaderBlue.ID, c_str!("Matrices").as_ptr());
        let uniformBlockIndexYellow = gl::GetUniformBlockIndex(shaderYellow.ID, c_str!("Matrices").as_ptr());
        // then we link each shader's uniform block to this uniform binding point
        gl::UniformBlockBinding(shaderRed.ID, uniformBlockIndexRed, 0);
        gl::UniformBlockBinding(shaderGreen.ID, uniformBlockIndexGreen, 0);
        gl::UniformBlockBinding(shaderBlue.ID, uniformBlockIndexBlue, 0);
        gl::UniformBlockBinding(shaderYellow.ID, uniformBlockIndexYellow, 0);
        // Now actually create the buffer
        let mut uboMatrices = 0;
        gl::GenBuffers(1, &mut uboMatrices);
        gl::BindBuffer(gl::UNIFORM_BUFFER, uboMatrices);
        gl::BufferData(gl::UNIFORM_BUFFER, 2 * mem::size_of::<Matrix4<f32>>() as isize, ptr::null(), gl::STATIC_DRAW);
        // define the range of the buffer that links to a uniform binding point
        gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, uboMatrices, 0, 2 * 2 * mem::size_of::<Matrix4<f32>>() as isize);

        // store the projection matrix (we only do this once now) (note: we're not using zoom anymore by changing the FoV)
        let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
        gl::BindBuffer(gl::UNIFORM_BUFFER, uboMatrices);
        gl::BufferSubData(gl::UNIFORM_BUFFER, 0, mem::size_of::<Matrix4<f32>>() as isize, projection.as_ptr() as *const c_void);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

        (shaderRed, shaderGreen, shaderBlue, shaderYellow, cubeVBO, cubeVAO, uboMatrices)
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
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // set the view and projection matrix in the uniform block - we only have to do this once per loop iteration.
            let view = camera.GetViewMatrix();
            gl::BindBuffer(gl::UNIFORM_BUFFER, uboMatrices);
            let size = mem::size_of::<Matrix4<f32>>() as isize;
            gl::BufferSubData(gl::UNIFORM_BUFFER, size, size, view.as_ptr() as *const c_void);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);

            // draw 4 cubes
            // RED
            gl::BindVertexArray(cubeVAO);
            shaderRed.useProgram();
            let mut model = Matrix4::from_translation(vec3(-0.75, 0.75, 0.0)); // move top-left
            shaderRed.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // GREEN
            shaderGreen.useProgram();
            model = Matrix4::from_translation(vec3(0.75, 0.75, 0.0)); // move top-right
            shaderGreen.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // YELLOW
            shaderYellow.useProgram();
            model = Matrix4::from_translation(vec3(-0.75, -0.75, 0.0)); // move bottom-left
            shaderYellow.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            // BLUE
            shaderBlue.useProgram();
            model = Matrix4::from_translation(vec3(0.75, -0.75, 0.0)); // move bottom-right
            shaderBlue.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
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
        gl::DeleteBuffers(1, &cubeVBO);
    }
}
