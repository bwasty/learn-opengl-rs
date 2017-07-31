#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
extern crate glfw;
use self::glfw::Context;

extern crate gl;
extern crate rand;
use self::rand::Rng;

use std::ffi::CStr;

use common::{process_events, processInput};
use shader::Shader;
use camera::Camera;
use model::Model;

use cgmath::{Matrix4, vec3, Point3, Deg, perspective};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_4_10_2() {
    let mut camera = Camera {
        Position: Point3::new(0.0, 0.0, 55.0),
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

    let (shader, rock, planet, modelMatrices) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile shaders
        // -------------------------
        let shader = Shader::new(
            "src/_4_advanced_opengl/shaders/10.2.instancing.vs",
            "src/_4_advanced_opengl/shaders/10.2.instancing.fs");

        // load models
        // -----------
        let rock = Model::new("resources/objects/rock/rock.obj");
        let planet = Model::new("resources/objects/planet/planet.obj");

        // generate a large list of semi-random model transformation matrices
        // ------------------------------------------------------------------
        let amount = 1000;
        let mut modelMatrices: Vec<Matrix4<f32>> = Vec::with_capacity(amount);
        let mut rng = rand::thread_rng();
        let radius = 50.0;
        let offset: f32 = 2.5;
        for i in 0..amount {
            let angle = i as i32 as f32 / amount as f32 * 360.0;
            let mut displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let x = angle.sin() * radius + displacement;
            displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let y = displacement * 0.4; // keep height of asteroid field smaller compared to width of x and z
            displacement = (rng.gen::<i32>() % (2.0 * offset * 100.0) as i32) as f32 / 100.0 - offset;
            let z = angle.cos() * radius + displacement;
            let mut model = Matrix4::<f32>::from_translation(vec3(x, y, z));

            // 2. scale: Scale between 0.05 and 0.25
            let scale = (rng.gen::<i32>() % 20) as f32 / 100.0 + 0.05;
            model = model * Matrix4::from_scale(scale);

            // 3. rotation: add random rotation around a (semi)randomly picked rotation axis vector
            let rotAngle = (rng.gen::<i32>() % 360) as f32;
            model = model * Matrix4::from_axis_angle(vec3(0.4, 0.6, 0.8).normalize(), Deg(rotAngle));

            // 4. now add to list of matrices
            modelMatrices.push(model);
        }

        (shader, rock, planet, modelMatrices)
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
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 1000.0);
            let view = camera.GetViewMatrix();
            shader.useProgram();
            shader.setMat4(c_str!("projection"), &projection);
            shader.setMat4(c_str!("view"), &view);

            // draw planet
            let mut model = Matrix4::<f32>::from_translation(vec3(0.0, -3.0, 0.0));
            model = model * Matrix4::from_scale(4.0);
            shader.setMat4(c_str!("model"), &model);
            planet.Draw(&shader);

            // draw meteorites
            for model in &modelMatrices {
                shader.setMat4(c_str!("model"), model);
                rock.Draw(&shader);
            }
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

}
