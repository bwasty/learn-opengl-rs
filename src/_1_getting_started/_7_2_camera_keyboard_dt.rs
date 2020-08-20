#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;
use self::gl::types::*;

use std::sync::mpsc::Receiver;
use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ffi::CStr;

use shader::Shader;

use image;
use image::GenericImageView;

use cgmath::{Matrix4, Vector3, vec3,  Deg, perspective, Point3};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

// camera
const cameraFront: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 0.0,
    z: -1.0,
};
const cameraUp: Vector3<f32> = Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

pub fn main_1_7_2() {
    let mut cameraPos = Point3::new(0.0, 0.0, 3.0);

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

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (ourShader, VBO, VAO, texture1, texture2, cubePositions) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile our shader program
        // ------------------------------------
        let ourShader = Shader::new(
            "src/_1_getting_started/shaders/7.2.camera.vs",
            "src/_1_getting_started/shaders/7.2.camera.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices: [f32; 180] = [
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
        // world space positions of our cubes
        let cubePositions: [Vector3<f32>; 10] = [vec3(0.0, 0.0, 0.0),
                                                 vec3(2.0, 5.0, -15.0),
                                                 vec3(-1.5, -2.2, -2.5),
                                                 vec3(-3.8, -2.0, -12.3),
                                                 vec3(2.4, -0.4, -3.5),
                                                 vec3(-1.7, 3.0, -7.5),
                                                 vec3(1.3, -2.0, -2.5),
                                                 vec3(1.5, 2.0, -2.5),
                                                 vec3(1.5, 0.2, -1.5),
                                                 vec3(-1.3, 1.0, -1.5)];
        let (mut VBO, mut VAO) = (0, 0);
        gl::GenVertexArrays(1, &mut VAO);
        gl::GenBuffers(1, &mut VBO);

        gl::BindVertexArray(VAO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        // position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        // texture coord attribute
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(1);


        // load and create a texture
        // -------------------------
        let (mut texture1, mut texture2) = (0, 0);
        // texture 1
        // ---------
        gl::GenTextures(1, &mut texture1);
        gl::BindTexture(gl::TEXTURE_2D, texture1);
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        // load image, create texture and generate mipmaps
        let img = image::open(&Path::new("resources/textures/container.jpg")).expect("Failed to load texture");
        let data = img.raw_pixels();
        let dim = img.dimensions();

        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       gl::RGB as i32,
                       dim.0 as i32,
                       dim.1 as i32,
                       0,
                       gl::RGB,
                       gl::UNSIGNED_BYTE,
                       &data[0] as *const u8 as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        // texture 2
        // ---------
        gl::GenTextures(1, &mut texture2);
        gl::BindTexture(gl::TEXTURE_2D, texture2);
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        // load image, create texture and generate mipmaps
        let img = image::open(&Path::new("resources/textures/awesomeface.png")).expect("Failed to load texture");
        let img = img.flipv(); // flip loaded texture on the y-axis.
        let data = img.raw_pixels();
        let dim = img.dimensions();
        
        // note that the awesomeface.png has transparency and thus an alpha channel, so make sure to tell OpenGL the data type is of GL_RGBA
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       gl::RGB as i32,
                       dim.0 as i32,
                       dim.1 as i32,
                       0,
                       gl::RGBA,
                       gl::UNSIGNED_BYTE,
                       &data[0] as *const u8 as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);

        // tell opengl for each sampler to which texture unit it belongs to (only has to be done once)
        // -------------------------------------------------------------------------------------------
        ourShader.useProgram();
        ourShader.setInt(c_str!("texture1"), 0);
        ourShader.setInt(c_str!("texture2"), 1);

        // pass projection matrix to shader (as projection matrix rarely changes there's no need to do this per frame)
        // -----------------------------------------------------------------------------------------------------------
        let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        ourShader.setMat4(c_str!("projection"), &projection);

        (ourShader, VBO, VAO, texture1, texture2, cubePositions)
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
        process_events(&events);

        // input
        // -----
        processInput(&mut window, deltaTime, &mut cameraPos);

        // render
        // ------
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // bind textures on corresponding texture units
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture2);

            // activate shader
            ourShader.useProgram();

            // camera/view transformation
            let view: Matrix4<f32> = Matrix4::look_at(cameraPos, cameraPos + cameraFront, cameraUp);
            ourShader.setMat4(c_str!("view"), &view);

            // render boxes
            gl::BindVertexArray(VAO);
            for (i, position) in cubePositions.iter().enumerate() {
                // calculate the model matrix for each object and pass it to shader before drawing
                let mut model: Matrix4<f32> = Matrix4::from_translation(*position);
                let angle = 20.0 * i as f32;
                model = model * Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg(angle));
                ourShader.setMat4(c_str!("model"), &model);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    unsafe {
        gl::DeleteVertexArrays(1, &VAO);
        gl::DeleteBuffers(1, &VBO);
    }
}

// NOTE: not the same version as in common.rs!
#[allow(unknown_lints)]
#[allow(single_match)]
fn process_events(events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            _ => {}
        }
    }
}

/// NOTE: not the same function as the one in common.rs!
fn processInput(window: &mut glfw::Window, deltaTime: f32, cameraPos: &mut Point3<f32>) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }

    let cameraSpeed = 2.5 * deltaTime;
    if window.get_key(Key::W) == Action::Press {
        *cameraPos += cameraSpeed * cameraFront;
    }
    if window.get_key(Key::S) == Action::Press {
        *cameraPos += -(cameraSpeed * cameraFront);
    }
    if window.get_key(Key::A) == Action::Press {
        *cameraPos += -(cameraFront.cross(cameraUp).normalize() * cameraSpeed);
    }
    if window.get_key(Key::D) == Action::Press {
        *cameraPos += cameraFront.cross(cameraUp).normalize() * cameraSpeed;
    }

}
