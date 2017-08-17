#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate glfw;
use self::glfw::Context;

extern crate gl;

extern crate num;

use std::ptr;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ffi::{CStr, CString};

use common::{process_events, processInput};
use shader::Shader;
use camera::Camera;

use cgmath::{Matrix4, vec3, Vector3, vec2, Deg, perspective, Point3};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_6_1_1() {
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
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
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

        // build and compile shaders
        // ------------------------------------
        let shader = Shader::new(
            "src/_6_pbr/shaders/1.1.pbr.vs",
            "src/_6_pbr/shaders/1.1.pbr.fs");

        shader.useProgram();
        shader.setVec3(c_str!("albedo"), 0.5, 0.0, 0.0);
        shader.setFloat(c_str!("ao"), 1.0);

        // initialize static shader uniforms before rendering
        // --------------------------------------------------
        let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
        shader.setMat4(c_str!("projection"), &projection);

        shader
    };

    // lights
    // ------
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

    let mut sphereVAO = 0;
    let mut indexCount = 0;

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
                        ((col - (nrColumns / 2)) as f32 * spacing),
                        ((row - (nrRows / 2)) as f32 * spacing),
                        0.0
                    ));
                    shader.setMat4(c_str!("model"), &model);
                    renderSphere(&mut sphereVAO, &mut indexCount);
                }
            }

            // render light source (simply re-render sphere at light positions)
            // this looks a bit off as we use the same shader, but it'll make their positions obvious and
            // keeps the codeprint small.
            for (i, lightPosition) in lightPositions.iter().enumerate() {
                // NOTE: toggle comments on next two lines to animate the lights
                // let newPos = lightPosition + vec3((glfw.get_time() as f32 * 5.0).sin() * 5.0, 0.0, 0.0);
                let newPos = *lightPosition;
                let mut name = CString::new(format!("lightPositions[{}]", i)).unwrap();
                shader.setVector3(&name, &newPos);
                name = CString::new(format!("lightColors[{}]", i)).unwrap();
                shader.setVector3(&name, &lightColors[i]);

                model = Matrix4::from_translation(newPos);
                model = model * Matrix4::from_scale(0.5);
                shader.setMat4(c_str!("model"), &model);
                renderSphere(&mut sphereVAO, &mut indexCount);
            }
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

}

pub unsafe fn renderSphere(sphereVAO: &mut u32, indexCount: &mut u32) {
    if *sphereVAO == 0 {
        gl::GenVertexArrays(1, sphereVAO);

        let mut vbo = 0;
        let mut ebo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        let mut positions = vec![];
        let mut uv = vec![];
        let mut normals = vec![];
        let mut indices = vec![];

        const X_SEGMENTS: u32 = 64;
        const Y_SEGMENTS: u32 = 64;
        const PI: f32 = 3.14159265359;
        for y in 0..Y_SEGMENTS+1 {
            for x in 0..X_SEGMENTS+1 {
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
                for x in 0..X_SEGMENTS+1 {
                    indices.push(y       * (X_SEGMENTS + 1) + x);
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                }
            }
            else {
                for x in (0..X_SEGMENTS+1).rev() {
                    indices.push((y + 1) * (X_SEGMENTS + 1) + x);
                    indices.push(y       * (X_SEGMENTS + 1) + x);
                }
            }
            oddRow = !oddRow;
        }
        *indexCount = indices.len() as u32;

        let mut data: Vec<f32> = Vec::new();
        for (i, position) in positions.iter().enumerate() {
            data.push(position.x);
            data.push(position.y);
            data.push(position.z);
            if uv.len() > 0 {
                data.push(uv[i].x);
                data.push(uv[i].y);
            }
            if normals.len() > 0 {
                data.push(normals[i].x);
                data.push(normals[i].y);
                data.push(normals[i].z);
            }
        }
        gl::BindVertexArray(*sphereVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * size_of::<f32>()) as isize,
            &data[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * size_of::<u32>()) as isize, &indices[0] as *const u32 as *const c_void, gl::STATIC_DRAW);
        let stride = (3 + 2 + 3) * size_of::<f32>() as i32;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * size_of::<f32>()) as *const c_void);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride, (5 * size_of::<f32>()) as *const c_void);
    }

    gl::BindVertexArray(*sphereVAO);
    gl::DrawElements(gl::TRIANGLE_STRIP, *indexCount as i32, gl::UNSIGNED_INT, ptr::null());
}
