#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;
use self::gl::types::*;

extern crate num;

use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ffi::{CStr, CString};

use image;
use image::GenericImage;
use image::DynamicImage::*;

use common::{process_events, processInput, loadTexture};
use shader::Shader;
use camera::Camera;
use camera::Camera_Movement::*;

use cgmath::{Matrix4, vec3, Vector3, vec2, Deg, perspective, Point3};
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

    let shader = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // build and compile shaders
        // ------------------------------------
        let shader = Shader::new(
            "src/_6_pbr/shaders/1.1.pbr.vs",
            "src/_6_pbr/shaders/1.1.pbr.fs");

        shader.useProgram();
        shader.setVec3(c_str!("albedo"), 0.5, 0.0, 0.0);
        shader.setFloat(c_str!("ao"), 1.0);

        // lights
        // -------------

        // initialize static shader uniforms before rendering
        // --------------------------------------------------
        // TODO!!!
        let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
        shader.setMat4(c_str!("projection"), &projection);

        shader
    };

    let lightPositions: [Vector3<f32>; 4] = [
        vec3(-10.0,  10.0, 10.0),
        vec3( 10.0,  10.0, 10.0),
        vec3(-10.0, -10.0, 10.0),
        vec3( 10.0, -10.0, 10.0)
    ];
    let lightColors: [Vector3<f32>; 4] = [
        vec3(300.0, 300.0, 300.0),
        vec3(300.0, 300.0, 300.0),
        vec3(300.0, 300.0, 300.0),
        vec3(300.0, 300.0, 300.0)
    ];
    let nrRows    = 7;
    let nrColumns = 7;
    let spacing = 2.5;


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

            // draw objects
            shader.useProgram();
            let view = camera.GetViewMatrix();
            shader.setMat4(c_str!("view"), &view);
            shader.setVector3(c_str!("camPos"), &camera.Position.to_vec());

            // render rows*column number of spheres with varying metallic/roughness values scaled by rows and columns respectively
            let mut model: Matrix4<f32>;
            for row in 0..nrRows {
                shader.setFloat(c_str!("metallic"), row as i32 as f32 / nrRows as f32);
                for col in 0..nrColumns {
                    // we clamp the roughness to 0.025 - 1.0 as perfectly smooth surfaces (roughness of 0.0) tend to look a bit off
                    // on direct lighting.
                    shader.setFloat(c_str!("roughness"), num::clamp(col as i32 as f32 / nrColumns as f32, 0.05, 1.0));

                    let model = Matrix4::from_translation(vec3(
                        (col as f32 - (nrColumns / 2) as f32 * spacing) as f32,
                        (row as f32 - (nrRows / 2) as f32 * spacing) as f32,
                        0.0
                    ));
                    shader.setMat4(c_str!("model"), &model);
                    renderSphere();
                }
            }

            // render light source (simply re-render sphere at light positions)
            // this looks a bit off as we use the same shader, but it'll make their positions obvious and
            // keeps the codeprint small.
            for (i, lightPosition) in lightPositions.iter().enumerate() {
                let mut newPos = lightPosition + vec3((glfw.get_time() as f32 * 5.0).sin() * 5.0, 0.0, 0.0);
                newPos = *lightPosition;
                let mut name = CString::new(format!("lightPositions[{}]", i)).unwrap();
                shader.setVector3(&name, &newPos);
                name = CString::new(format!("lightColors[{}]", i)).unwrap();
                shader.setVector3(&name, &lightColors[i]);

                model = Matrix4::from_translation(newPos);
                model = model * Matrix4::from_scale(0.5);
                shader.setMat4(c_str!("model"), &model);
                renderSphere();
            }
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

}

pub fn renderSphere(sphereVAO: &mut u32, indexCount: u32) {
    if *sphereVAO == 0 {
        gl::GenVertexArrays(1, sphereVAO);

        let mut vbo = 0;
        let mut ebo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        let positions = vec![];
        let uv = vec![];
        let normals = vec![];
        let indices = vec![];

        const X_SEGMENTS: u32 = 64;
        const Y_SEGMENTS: u32 = 64;
        const PI: f32 = 3.14159265359;
        for y in 0..Y_SEGMENTS {
            for x in 0..X_SEGMENTS {
                let xSegment = x as f32 / X_SEGMENTS as f32;
                let ySegment = y as f32 / Y_SEGMENTS as f32;
                let xPos = (xSegment * 2.0 * PI).cos() * (ySegment * PI).sin();
                let yPos = (ySegment * PI).cos();
                let zPos = (xSegment * 2.0 * PI).sin() * (ySegment * PI).sin();

                positions.push(vec3(xPos, yPos, zPos));
                uv.push(vec2(xSegment, ySegment));
                normals.push(vec3(xPos, yPos, zPos));
            }
        }

        let mut oddRow = false;
        for y in 0..Y_SEGMENTS {
            if oddRow { // even rows: y == 0, y == 2; and so on
                for x in 0..X_SEGMENTS {
                    indices.push(y       * (X_SEGMENTS + 1) + x);
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                }
            }
            else {
                for x in 0..X_SEGMENTS {
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                    indices.push(y       * (X_SEGMENTS + 1) + x);
                }
            }
            oddRow = !oddRow;
        }
        indexCount = indices.len() as u32;

        // TODO!!!
    }
}
