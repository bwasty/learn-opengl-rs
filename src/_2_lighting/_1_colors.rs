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

use common::{process_events, processInput};
use shader::Shader;
use camera::Camera;

use cgmath::{Matrix4, vec3, Point3, Deg, perspective};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

pub fn main_2_1() {
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

    // lighting
    let lightPos = vec3(1.2, 1.0, 2.0);

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

    let (lightingShader, lampShader, VBO, cubeVAO, lightVAO) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile our shader program
        // ------------------------------------
        let lightingShader = Shader::new("src/_2_lighting/shaders/1.colors.vs", "src/_2_lighting/shaders/1.colors.fs");
        let lampShader = Shader::new("src/_2_lighting/shaders/1.lamp.vs", "src/_2_lighting/shaders/1.lamp.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices: [f32; 108] = [
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
        // first, configure the cube's VAO (and VBO)
        let (mut VBO, mut cubeVAO) = (0, 0);
        gl::GenVertexArrays(1, &mut cubeVAO);
        gl::GenBuffers(1, &mut VBO);

        gl::BindVertexArray(cubeVAO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        let stride = 3 * mem::size_of::<GLfloat>() as GLsizei;
        // position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // second, configure the light's VAO (VBO stays the same; the vertices are the same for the light object which is also a 3D cube)
        let mut lightVAO = 0;
        gl::GenVertexArrays(1, &mut lightVAO);
        gl::BindVertexArray(lightVAO);

        // we only need to bind to the VBO (to link it with glVertexAttribPointer), no need to fill it; the VBO's data already contains all we need (it's already bound, but we do it again for educational purposes)
        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        (lightingShader, lampShader, VBO, cubeVAO, lightVAO)
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
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // be sure to activate shader when setting uniforms/drawing objects
            lightingShader.useProgram();
            lightingShader.setVec3(c_str!("objectColor"), 1.0, 0.5, 0.31);
            lightingShader.setVec3(c_str!("lightColor"), 1.0, 1.0, 1.0);

            // view/projection transformations
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.GetViewMatrix();
            lightingShader.setMat4(c_str!("projection"), &projection);
            lightingShader.setMat4(c_str!("view"), &view);

            // world transformation
            let mut model = Matrix4::<f32>::identity();
            lightingShader.setMat4(c_str!("model"), &model);

            // render the cube
            gl::BindVertexArray(cubeVAO);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);


            // also draw the lamp object
            lampShader.useProgram();
            lampShader.setMat4(c_str!("projection"), &projection);
            lampShader.setMat4(c_str!("view"), &view);
            model = Matrix4::from_translation(lightPos);
            model = model * Matrix4::from_scale(0.2);  // a smaller cube
            lampShader.setMat4(c_str!("model"), &model);

            gl::BindVertexArray(lightVAO);
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
        gl::DeleteVertexArrays(1, &lightVAO);
        gl::DeleteBuffers(1, &VBO);
    }
}
