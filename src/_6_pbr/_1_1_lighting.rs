#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;
use self::gl::types::*;

use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ffi::CStr;

use image;
use image::GenericImage;
use image::DynamicImage::*;

use common::process_events;
use shader::Shader;
use camera::Camera;
use camera::Camera_Movement::*;

use cgmath::{Matrix4, vec3, Vector3, Deg, perspective, Point3};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

// TODO!!: copied from 5.2
pub fn main_6_1_1() {
    let mut gammaEnabled = false;
    let mut gammaKeyPressed = false;

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

    let (shader, planeVBO, planeVAO, floorTexture, floorTextureGammaCorrected) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // build and compile shaders
        // ------------------------------------
        let shader = Shader::new(
            "src/_5_advanced_lighting/shaders/2.gamma_correction.vs",
            "src/_5_advanced_lighting/shaders/2.gamma_correction.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let planeVertices: [f32; 48] = [
            // positions         // normals      // texcoords
             10.0, -0.5,  10.0,  0.0, 1.0, 0.0,  10.0,  0.0,
            -10.0, -0.5,  10.0,  0.0, 1.0, 0.0,   0.0,  0.0,
            -10.0, -0.5, -10.0,  0.0, 1.0, 0.0,   0.0, 10.0,

             10.0, -0.5,  10.0,  0.0, 1.0, 0.0,  10.0,  0.0,
            -10.0, -0.5, -10.0,  0.0, 1.0, 0.0,   0.0, 10.0,
             10.0, -0.5, -10.0,  0.0, 1.0, 0.0,  10.0, 10.0
        ];
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
        let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::BindVertexArray(0);

        // load textures
        // -------------
        let floorTexture = loadTexture("resources/textures/wood.png", false);
        let floorTextureGammaCorrected = loadTexture("resources/textures/wood.png", true);

        // shader configuration
        // --------------------
        shader.useProgram();
        shader.setInt(c_str!("floorTexture"), 0);

        (shader, planeVBO, planeVAO, floorTexture, floorTextureGammaCorrected)
    };

    // lighting info
    // -------------
    let lightPositions: [Vector3<f32>; 4] = [
        vec3(-3.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3 (1.0, 0.0, 0.0),
        vec3 (3.0, 0.0, 0.0)
    ];
    let lightColors: [Vector3<f32>; 4] = [
        vec3(0.25, 0.25, 0.25),
        vec3(0.50, 0.50, 0.50),
        vec3(0.75, 0.75, 0.75),
        vec3(1.00, 1.00, 1.00)
    ];

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
        processInput(&mut window, deltaTime, &mut camera, &mut gammaEnabled, &mut gammaKeyPressed);

        // render
        // ------
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // draw objects
            shader.useProgram();
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
            let view = camera.GetViewMatrix();
            shader.setMat4(c_str!("projection"), &projection);
            shader.setMat4(c_str!("view"), &view);
            // set light uniforms
            shader.setVector3(c_str!("lightPositions[0]"), &lightPositions[0]);
            shader.setVector3(c_str!("lightPositions[1]"), &lightPositions[1]);
            shader.setVector3(c_str!("lightPositions[2]"), &lightPositions[2]);
            shader.setVector3(c_str!("lightPositions[3]"), &lightPositions[3]);
            shader.setVector3(c_str!("lightColors[0]"), &lightColors[0]);
            shader.setVector3(c_str!("lightColors[1]"), &lightColors[1]);
            shader.setVector3(c_str!("lightColors[2]"), &lightColors[2]);
            shader.setVector3(c_str!("lightColors[3]"), &lightColors[3]);
            shader.setVector3(c_str!("viewPos"), &camera.Position.to_vec());
            shader.setBool(c_str!("gamma"), gammaEnabled);
            // floor
            gl::BindVertexArray(planeVAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, if gammaEnabled { floorTextureGammaCorrected } else { floorTexture });
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    unsafe {
        gl::DeleteVertexArrays(1, &planeVAO);
        gl::DeleteBuffers(1, &planeVBO);
    }
}

// NOTE: not the same version as in common.rs
pub fn processInput(window: &mut glfw::Window, deltaTime: f32, camera: &mut Camera, gammaEnabled: &mut bool, gammaKeyPressed: &mut bool) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }

    if window.get_key(Key::W) == Action::Press {
        camera.ProcessKeyboard(FORWARD, deltaTime);
    }
    if window.get_key(Key::S) == Action::Press {
        camera.ProcessKeyboard(BACKWARD, deltaTime);
    }
    if window.get_key(Key::A) == Action::Press {
        camera.ProcessKeyboard(LEFT, deltaTime);
    }
    if window.get_key(Key::D) == Action::Press {
        camera.ProcessKeyboard(RIGHT, deltaTime);
    }

    if window.get_key(Key::Space) == Action::Press && !(*gammaKeyPressed) {
        *gammaEnabled = !(*gammaEnabled);
        *gammaKeyPressed = true;
        println!("{}", if *gammaEnabled { "Gamma Enabled" } else { "Gamma disabled" })
    }
    if window.get_key(Key::Space) == Action::Release {
        *gammaKeyPressed = false;
    }
}

// NOTE: not the same version as in common.rs
pub unsafe fn loadTexture(path: &str, gammaCorrection: bool) -> u32 {
    let mut textureID = 0;

    gl::GenTextures(1, &mut textureID);
    let img = image::open(&Path::new(path)).expect("Texture failed to load");
    // need two different formats for gamma correction
    let (internalFormat, dataFormat) = match img {
        ImageLuma8(_) => (gl::RED, gl::RED),
        ImageLumaA8(_) => (gl::RG, gl::RG),
        ImageRgb8(_) => (if gammaCorrection { gl::SRGB } else { gl::RGB }, gl::RGB),
        ImageRgba8(_) => (if gammaCorrection { gl::SRGB_ALPHA } else { gl::RGB }, gl::RGBA),
    };

    let data = img.raw_pixels();

    gl::BindTexture(gl::TEXTURE_2D, textureID);
    gl::TexImage2D(gl::TEXTURE_2D, 0, internalFormat as i32, img.width() as i32, img.height() as i32,
        0, dataFormat, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);
    gl::GenerateMipmap(gl::TEXTURE_2D);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    textureID
}
