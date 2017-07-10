#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::ffi::CStr;

extern crate glfw;
use self::glfw::Context;

extern crate gl;

use cgmath::{Matrix4, Deg, vec3, Point3, perspective};

use common::{process_events, processInput};
use shader::Shader;
use camera::Camera;
use model::Model;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_4_9_3() {
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

    let (shader, normalShader, nanoSuit) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile shaders
        // -------------------------
        let shader = Shader::new(
            "src/_4_advanced_opengl/shaders/9.3.default.vs",
            "src/_4_advanced_opengl/shaders/9.3.default.fs",
        );
        let normalShader = Shader::with_geometry_shader(
            "src/_4_advanced_opengl/shaders/9.3.normal_visualization.vs",
            "src/_4_advanced_opengl/shaders/9.3.normal_visualization.fs",
            "src/_4_advanced_opengl/shaders/9.3.normal_visualization.gs"
        );

        // load models
        // -----------
        let nanoSuit = Model::new("resources/objects/nanosuit/nanosuit.obj");

        (shader, normalShader, nanoSuit)
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

            // configure transformation matrices
            let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.GetViewMatrix();
            let mut model = Matrix4::<f32>::from_translation(vec3(0.0, -1.75, 0.0)); // translate it down so it's at the center of the scene
            model = model * Matrix4::from_scale(0.2);  // it's a bit too big for our scene, so scale it down
            shader.useProgram();
            shader.setMat4(c_str!("projection"), &projection);
            shader.setMat4(c_str!("view"), &view);
            shader.setMat4(c_str!("model"), &model);

            // draw model as usual
            nanoSuit.Draw(&shader);

            // then draw model with normal visualizing geometry shader
            normalShader.useProgram();
            normalShader.setMat4(c_str!("projection"), &projection);
            normalShader.setMat4(c_str!("view"), &view);
            normalShader.setMat4(c_str!("model"), &model);

            nanoSuit.Draw(&normalShader);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}
