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

use cgmath::{Matrix4, vec3, Vector3, vec2, Vector2, Deg, perspective, Point3};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_5_4() {
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

    let (shader, diffuseMap, normalMap) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile shaders
        // ------------------------------------
        let shader = Shader::new(
            "src/_5_advanced_lighting/shaders/4.normal_mapping.vs",
            "src/_5_advanced_lighting/shaders/4.normal_mapping.fs");

        // load textures
        // -------------
        let diffuseMap = loadTexture("resources/textures/brickwall.jpg");
        let normalMap = loadTexture("resources/textures/brickwall_normal.jpg");

        // shader configuration
        // --------------------
        shader.useProgram();
        shader.setInt(c_str!("diffuseMap"), 0);
        shader.setInt(c_str!("normalMap"), 1);

        (shader, diffuseMap, normalMap)
    };

    // lighting info
    // -------------
    let lightPos: Vector3<f32> = vec3(0.5, 1.0, 0.3);

    let mut quadVAO = 0;
    let mut quadVBO = 0;

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

             // configure view/projection matrices
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
            let view = camera.GetViewMatrix();
            shader.useProgram();
            shader.setMat4(c_str!("projection"), &projection);
            shader.setMat4(c_str!("view"), &view);
            // render normal-mapped quad
            let mut model: Matrix4<f32> = Matrix4::from_axis_angle(vec3(1.0, 0.0, 1.0).normalize(), Deg(glfw.get_time() as f32 * -10.0));// rotate the quad to show normal mapping from multiple directions
            shader.setMat4(c_str!("model"), &model);
            shader.setVector3(c_str!("viewPos"), &camera.Position.to_vec());
            shader.setVector3(c_str!("lightPos"), &lightPos);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, diffuseMap);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, normalMap);
            renderQuad(&mut quadVAO, &mut quadVBO);

            // render light source (simply re-renders a smaller plane at the light's position for debugging/visualization)
            model = Matrix4::from_translation(lightPos);
            model = model * Matrix4::from_scale(0.1);
            shader.setMat4(c_str!("model"), &model);
            renderQuad(&mut quadVAO, &mut quadVBO);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

// renders a 1x1 quad in NDC with manually calculated tangent vectors
// ------------------------------------------------------------------
unsafe fn renderQuad(quadVAO: &mut u32, quadVBO: &mut u32) {
    if *quadVAO == 0 {
        // positions
        let pos1: Vector3<f32> = vec3(-1.0,  1.0, 0.0);
        let pos2: Vector3<f32> = vec3(-1.0, -1.0, 0.0);
        let pos3: Vector3<f32> = vec3( 1.0, -1.0, 0.0);
        let pos4: Vector3<f32> = vec3( 1.0,  1.0, 0.0);
        // texture coordinates
        let uv1: Vector2<f32> = vec2(0.0, 1.0);
        let uv2: Vector2<f32> = vec2(0.0, 0.0);
        let uv3: Vector2<f32> = vec2(1.0, 0.0);
        let uv4: Vector2<f32> = vec2(1.0, 1.0);
        // normal vector
        let nm: Vector3<f32> = vec3(0.0, 0.0, 1.0);

        // calculate tangent/bitangent vectors of both triangles
        let mut tangent1: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        let mut bitangent1: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        let mut tangent2: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        let mut bitangent2: Vector3<f32> = vec3(0.0, 0.0, 0.0);
        // triangle 1
        // ----------
        let mut edge1 = pos2 - pos1;
        let mut edge2 = pos3 - pos1;
        let mut deltaUV1 = uv2 - uv1;
        let mut deltaUV2 = uv3 - uv1;

        let mut f = 1.0 / (deltaUV1.x * deltaUV2.y - deltaUV2.x * deltaUV1.y);

        tangent1.x = f * (deltaUV2.y * edge1.x - deltaUV1.y * edge2.x);
        tangent1.y = f * (deltaUV2.y * edge1.y - deltaUV1.y * edge2.y);
        tangent1.z = f * (deltaUV2.y * edge1.z - deltaUV1.y * edge2.z);
        tangent1 = tangent1.normalize();

        bitangent1.x = f * (-deltaUV2.x * edge1.x + deltaUV1.x * edge2.x);
        bitangent1.y = f * (-deltaUV2.x * edge1.y + deltaUV1.x * edge2.y);
        bitangent1.z = f * (-deltaUV2.x * edge1.z + deltaUV1.x * edge2.z);
        bitangent1 = bitangent1.normalize();

        // triangle 2
        // ----------
        edge1 = pos3 - pos1;
        edge2 = pos4 - pos1;
        deltaUV1 = uv3 - uv1;
        deltaUV2 = uv4 - uv1;

        f = 1.0 / (deltaUV1.x * deltaUV2.y - deltaUV2.x * deltaUV1.y);

        tangent2.x = f * (deltaUV2.y * edge1.x - deltaUV1.y * edge2.x);
        tangent2.y = f * (deltaUV2.y * edge1.y - deltaUV1.y * edge2.y);
        tangent2.z = f * (deltaUV2.y * edge1.z - deltaUV1.y * edge2.z);
        tangent2 = tangent2.normalize();

        bitangent2.x = f * (-deltaUV2.x * edge1.x + deltaUV1.x * edge2.x);
        bitangent2.y = f * (-deltaUV2.x * edge1.y + deltaUV1.x * edge2.y);
        bitangent2.z = f * (-deltaUV2.x * edge1.z + deltaUV1.x * edge2.z);
        bitangent2 = bitangent2.normalize();

        let quadVertices: [f32; 84] = [
            // positions            // normal         // texcoords  // tangent                          // bitangent
            pos1.x, pos1.y, pos1.z, nm.x, nm.y, nm.z, uv1.x, uv1.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,
            pos2.x, pos2.y, pos2.z, nm.x, nm.y, nm.z, uv2.x, uv2.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,
            pos3.x, pos3.y, pos3.z, nm.x, nm.y, nm.z, uv3.x, uv3.y, tangent1.x, tangent1.y, tangent1.z, bitangent1.x, bitangent1.y, bitangent1.z,

            pos1.x, pos1.y, pos1.z, nm.x, nm.y, nm.z, uv1.x, uv1.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z,
            pos3.x, pos3.y, pos3.z, nm.x, nm.y, nm.z, uv3.x, uv3.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z,
            pos4.x, pos4.y, pos4.z, nm.x, nm.y, nm.z, uv4.x, uv4.y, tangent2.x, tangent2.y, tangent2.z, bitangent2.x, bitangent2.y, bitangent2.z
        ];

        // configure plane VAO
        gl::GenVertexArrays(1, quadVAO);
        gl::GenBuffers(1, quadVBO);
        gl::BindVertexArray(*quadVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, *quadVBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (quadVertices.len() * mem::size_of::<f32>()) as isize,
            &quadVertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW);
        let stride = 14 * mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, stride, (8 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, stride, (11 * mem::size_of::<GLfloat>()) as *const c_void);
    }

    gl::BindVertexArray(*quadVAO);
    gl::DrawArrays(gl::TRIANGLES, 0, 6);
    gl::BindVertexArray(0);
}
