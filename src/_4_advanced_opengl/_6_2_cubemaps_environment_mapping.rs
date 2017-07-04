#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ffi::CStr;

extern crate glfw;
use self::glfw::Context;

extern crate gl;
use self::gl::types::*;

use cgmath::{Matrix4,  Deg, perspective, Point3};
use cgmath::prelude::*;

use image;
use image::GenericImage;

use common::{process_events, processInput};
use shader::Shader;
use camera::Camera;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_4_6_2() {
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

    let (shader, skyboxShader, cubeVBO, cubeVAO, skyboxVBO, skyboxVAO, cubemapTexture) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::ALWAYS); // always pass the depth test (same effect as glDisable(GL_DEPTH_TEST))

        // build and compile our shader program
        // ------------------------------------
        let shader = Shader::new(
            "src/_4_advanced_opengl/shaders/6.2.cubemaps.vs",
            "src/_4_advanced_opengl/shaders/6.2.cubemaps.fs");
        let skyboxShader = Shader::new(
            "src/_4_advanced_opengl/shaders/6.2.skybox.vs",
            "src/_4_advanced_opengl/shaders/6.2.skybox.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let cubeVertices: [f32; 216] = [
            // positions       // normals
            -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
             0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
             0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
             0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
            -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
            -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,

            -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,
             0.5, -0.5,  0.5,  0.0,  0.0, 1.0,
             0.5,  0.5,  0.5,  0.0,  0.0, 1.0,
             0.5,  0.5,  0.5,  0.0,  0.0, 1.0,
            -0.5,  0.5,  0.5,  0.0,  0.0, 1.0,
            -0.5, -0.5,  0.5,  0.0,  0.0, 1.0,

            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,
            -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
            -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,
            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,

             0.5,  0.5,  0.5,  1.0,  0.0,  0.0,
             0.5,  0.5, -0.5,  1.0,  0.0,  0.0,
             0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
             0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
             0.5, -0.5,  0.5,  1.0,  0.0,  0.0,
             0.5,  0.5,  0.5,  1.0,  0.0,  0.0,

            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
             0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
             0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
             0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
            -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,

            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
             0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
             0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
             0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
            -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0
        ];
        let skyboxVertices: [f32; 108] = [
            // positions
            -1.0,  1.0, -1.0,
            -1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,

            -1.0, -1.0,  1.0,
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
            -1.0, -1.0,  1.0,

             1.0, -1.0, -1.0,
             1.0, -1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0, -1.0,
             1.0, -1.0, -1.0,

            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,

            -1.0,  1.0, -1.0,
             1.0,  1.0, -1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0,

            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0,  1.0
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
        let mut stride = 6 * mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        // skybox VAO
        let (mut skyboxVAO, mut skyboxVBO) = (0, 0);
        gl::GenVertexArrays(1, &mut skyboxVAO);
        gl::GenBuffers(1, &mut skyboxVBO);
        gl::BindVertexArray(skyboxVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, skyboxVBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (skyboxVertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &skyboxVertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        stride = 3 * mem::size_of::<GLfloat>() as GLsizei;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());

        // load textures
        // -------------
        let faces = [
            "resources/textures/skybox/right.jpg",
            "resources/textures/skybox/left.jpg",
            "resources/textures/skybox/top.jpg",
            "resources/textures/skybox/bottom.jpg",
            "resources/textures/skybox/back.jpg",
            "resources/textures/skybox/front.jpg"
        ];
        let cubemapTexture = loadCubemap(&faces);

        // shader configuration
        // --------------------
        shader.useProgram();
        shader.setInt(c_str!("skybox"), 0);

        skyboxShader.useProgram();
        skyboxShader.setInt(c_str!("skybox"), 0);

        (shader, skyboxShader, cubeVBO, cubeVAO, skyboxVBO, skyboxVAO, cubemapTexture)
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

            shader.useProgram();
            let model: Matrix4<f32> = Matrix4::identity();
            let mut view = camera.GetViewMatrix();
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
            shader.setMat4(c_str!("model"), &model);
            shader.setMat4(c_str!("view"), &view);
            shader.setMat4(c_str!("projection"), &projection);
            shader.setVector3(c_str!("cameraPos"), &camera.Position.to_vec());
            // cubes
            gl::BindVertexArray(cubeVAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, cubemapTexture);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);

            // draw skybox as last
            gl::DepthFunc(gl::LEQUAL);  // change depth function so depth test passes when values are equal to depth buffer's content
            skyboxShader.useProgram();
            // remove translation from the view matrix
            view = camera.GetViewMatrix();
            view.w[0] = 0.0;
            view.w[1] = 0.0;
            view.w[2] = 0.0;
            skyboxShader.setMat4(c_str!("view"), &view);
            skyboxShader.setMat4(c_str!("projection"), &projection);
            // skybox cube
            gl::BindVertexArray(skyboxVAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemapTexture);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);
            gl::DepthFunc(gl::LESS); // set depth function back to default
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
        gl::DeleteVertexArrays(1, &skyboxVAO);
        gl::DeleteBuffers(1, &cubeVBO);
        gl::DeleteBuffers(1, &skyboxVBO);
    }
}

/// loads a cubemap texture from 6 individual texture faces
/// order:
/// +X (right)
/// -X (left)
/// +Y (top)
/// -Y (bottom)
/// +Z (front)
/// -Z (back)
/// -------------------------------------------------------
unsafe fn loadCubemap(faces: &[&str]) -> u32 {
    let mut textureID = 0;
    gl::GenTextures(1, &mut textureID);
    gl::BindTexture(gl::TEXTURE_CUBE_MAP, textureID);

    for (i, face) in faces.iter().enumerate() {
        let img = image::open(&Path::new(face)).expect("Cubemap texture failed to load");

        let data = img.raw_pixels();
        gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
            0, gl::RGB as i32, img.width() as i32, img.height() as i32,
            0, gl::RGB, gl::UNSIGNED_BYTE,
            &data[0] as *const u8 as *const c_void);
    }

    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);

    textureID
}
