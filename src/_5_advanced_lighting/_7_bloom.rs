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
use std::ffi::{CStr, CString};

use image;
use image::GenericImage;
use image::DynamicImage::*;

use common::{process_events};
use shader::Shader;
use camera::Camera;
use camera::Camera_Movement::*;

use cgmath::{Matrix4, vec3, Vector3, Deg, perspective, Point3};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_5_7() {
    let mut bloom = true;
    let mut bloomKeyPressed = false;
    let mut exposure: f32 = 1.0;

    let mut camera = Camera {
        Position: Point3::new(0.0, 0.0, 5.0),
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

    let mut cubeVAO = 0;
    let mut cubeVBO = 0;
    let (shader, shaderLight, shaderBlur, shaderBloomFinal, woodTexture, containerTexture, hdrFBO, pingpongFBO,
        colorBuffers, pingpongColorbuffers, lightPositions, lightColors) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile shaders
        // ------------------------------------
        let shader = Shader::new(
            "src/_5_advanced_lighting/shaders/7.bloom.vs",
            "src/_5_advanced_lighting/shaders/7.bloom.fs");
        let shaderLight = Shader::new(
            "src/_5_advanced_lighting/shaders/7.bloom.vs",
            "src/_5_advanced_lighting/shaders/7.light_box.fs");
        let shaderBlur = Shader::new(
            "src/_5_advanced_lighting/shaders/7.blur.vs",
            "src/_5_advanced_lighting/shaders/7.blur.fs");
        let shaderBloomFinal = Shader::new(
            "src/_5_advanced_lighting/shaders/7.bloom_final.vs",
            "src/_5_advanced_lighting/shaders/7.bloom_final.fs");

        // load textures
        // -------------
        let woodTexture = loadTexture("resources/textures/wood.png", true); // note that we're loading the texture as an SRGB texture
        let containerTexture = loadTexture("resources/textures/container2.png", true); // note that we're loading the texture as an SRGB texture

        // configure (floating point) framebuffers
        // ---------------------------------------
        let mut hdrFBO = 0;
        gl::GenFramebuffers(1, &mut hdrFBO);     
        gl::BindFramebuffer(gl::FRAMEBUFFER, hdrFBO);
        // create 2 floating point color buffers (1 for normal rendering, other for brightness threshold values)
        let mut colorBuffers = [0; 2];
        gl::GenTextures(2, colorBuffers.as_mut_ptr());
        for (i, &colorBuffer) in colorBuffers.iter().enumerate() {
            gl::BindTexture(gl::TEXTURE_2D, colorBuffer);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB16F as i32, SCR_WIDTH as i32, SCR_HEIGHT as i32, 0,
                gl::RGB, gl::FLOAT, ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);  // we clamp to the edge as the blur filter would otherwise sample repeated texture values!
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            // attach texture to framebuffer
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as u32, gl::TEXTURE_2D,
                colorBuffer, 0);
        }
        // create and attach depth buffer (renderbuffer)
        let mut rboDepth = 0;
        gl::GenRenderbuffers(1, &mut rboDepth);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rboDepth);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, SCR_WIDTH as i32, SCR_HEIGHT as i32);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rboDepth);
        // tell OpenGL which color attachments we'll use (of this framebuffer) for rendering 
        let mut attachments = [ gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1 ];
        gl::DrawBuffers(2, attachments.as_mut_ptr());
        // finally check if framebuffer is complete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("Framebuffer not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // ping-pong-framebuffer for blurring
        let mut pingpongFBO = [0; 2];
        let mut pingpongColorbuffers = [0; 2];
        gl::GenFramebuffers(2, pingpongFBO.as_mut_ptr());
        gl::GenTextures(2, pingpongColorbuffers.as_mut_ptr());
        for i in 0..2 {
            gl::BindFramebuffer(gl::FRAMEBUFFER, pingpongFBO[i]);
            gl::BindTexture(gl::TEXTURE_2D, pingpongColorbuffers[i]);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB16F as i32, SCR_WIDTH as i32, SCR_HEIGHT as i32, 0, gl::RGB,
                gl::FLOAT, ptr::null());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32); // we clamp to the edge as the blur filter would otherwise sample repeated texture values!
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, 
                pingpongColorbuffers[i], 0);
            // also check if framebuffers are complete (no need for depth buffer)
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer not complete!");
            }
        }

        // lighting info
        // -------------
        // positions
        let mut lightPositions: Vec<Vector3<f32>> = Vec::new();
        lightPositions.push(vec3(  0.0,  0.5,  1.5));
        lightPositions.push(vec3( -4.0,  0.5, -3.0));
        lightPositions.push(vec3(  3.0,  0.5,  1.0));
        lightPositions.push(vec3( -0.8,  2.4, -1.0));
        // colors
        let mut lightColors: Vec<Vector3<f32>> = Vec::new();
        lightColors.push(vec3( 5.0,  5.0,  5.0));
        lightColors.push(vec3(10.0,  0.0,  0.0));
        lightColors.push(vec3( 0.0,  0.0, 15.0));
        lightColors.push(vec3( 0.0,  5.0,  0.0));

        
        // shader configuration
        // --------------------
        shader.useProgram();
        shader.setInt(c_str!("diffuseTexture"), 0);
        shaderBlur.useProgram();
        shaderBlur.setInt(c_str!("image"), 0);
        shaderBloomFinal.useProgram();
        shaderBloomFinal.setInt(c_str!("scene"), 0);
        shaderBloomFinal.setInt(c_str!("bloomBlur"), 1);
        
        (shader, shaderLight, shaderBlur, shaderBloomFinal, woodTexture, containerTexture, hdrFBO, pingpongFBO,
            colorBuffers, pingpongColorbuffers, lightPositions, lightColors)
    };

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
        processInput(&mut window, deltaTime, &mut camera, &mut bloom, &mut bloomKeyPressed, &mut exposure);

        // render
        // ------
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // 1. render scene into floating point framebuffer
            // -----------------------------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, hdrFBO);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32,
                0.1, 100.0);
            let view = camera.GetViewMatrix();
            shader.useProgram();
            shader.setMat4(c_str!("projection"), &projection);
            shader.setMat4(c_str!("view"), &view);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, woodTexture);
            // set lighting uniforms
            for (i, lightPos) in lightPositions.iter().enumerate() {
                let name = CString::new(format!("lights[{}].Position", i)).unwrap();
                shader.setVector3(&name, lightPos);
                let name = CString::new(format!("lights[{}].Color", i)).unwrap();
                shader.setVector3(&name, &lightColors[i]);
            }
            shader.setVector3(c_str!("viewPos"), &camera.Position.to_vec());
            // create one large cube that acts as the floor
            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, -1.0, 0.0));
            model = model * Matrix4::from_nonuniform_scale(12.5, 0.5, 12.5);
            shader.setMat4(c_str!("model"), &model);
            renderCube(&mut cubeVAO, &mut cubeVBO);
            // then create multiple cubes as the scenery
            gl::BindTexture(gl::TEXTURE_2D, containerTexture);
            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, 1.5, 0.0));
            model = model * Matrix4::from_scale(0.5);
            shader.setMat4(c_str!("model"), &model);
            renderCube(&mut cubeVAO, &mut cubeVBO);

            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(2.0, 0.0, 1.0));
            model = model * Matrix4::from_scale(0.5);
            shader.setMat4(c_str!("model"), &model);
            renderCube(&mut cubeVAO, &mut cubeVBO);

            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(-1.0, -1.0, 2.0));
            model = model * Matrix4::from_axis_angle(vec3(1.0, 0.0, 1.0).normalize(), Deg(60.0));
            shader.setMat4(c_str!("model"), &model);
            renderCube(&mut cubeVAO, &mut cubeVBO);

            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, 2.7, 4.0));
            model = model * Matrix4::from_axis_angle(vec3(1.0, 0.0, 1.0).normalize(), Deg(23.0));
            model = model * Matrix4::from_scale(1.25);
            shader.setMat4(c_str!("model"), &model);
            renderCube(&mut cubeVAO, &mut cubeVBO);

            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(-2.0, 1.0, -3.0));
            model = model * Matrix4::from_axis_angle(vec3(1.0, 0.0, 1.0).normalize(), Deg(124.0));
            shader.setMat4(c_str!("model"), &model);
            renderCube(&mut cubeVAO, &mut cubeVBO);

            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(-3.0, 0.0, 0.0));
            model = model * Matrix4::from_scale(0.5);
            shader.setMat4(c_str!("model"), &model);
            renderCube(&mut cubeVAO, &mut cubeVBO);

            // finally show all the light sources as bright cubes
            shaderLight.useProgram();
            shaderLight.setMat4(c_str!("projection"),&projection);
            shaderLight.setMat4(c_str!("view"), &view);

            for (lightPos, lightColor) in lightPositions.iter().zip(lightColors.iter()) {
                let mut model: Matrix4<f32> = Matrix4::from_translation(*lightPos);
                model = model * Matrix4::from_scale(0.25);
                shaderLight.setMat4(c_str!("model"), &model);
                shaderLight.setVector3(c_str!("lightColor"), lightColor);
                renderCube(&mut cubeVAO, &mut cubeVBO);
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 2. blur bright fragments with two-pass Gaussian Blur 
            // --------------------------------------------------
            let mut horizontal = true;
            let mut first_iteration = true; 
            let amount = 10;
            shaderBlur.useProgram();
            for _ in 0..amount {
                gl::BindFramebuffer(gl::FRAMEBUFFER, pingpongFBO[horizontal as usize]);
                shaderBlur.setInt(c_str!("horizontal"), horizontal as i32);
                gl::BindTexture(gl::TEXTURE_2D,
                    if first_iteration { colorBuffers[1] } else { pingpongColorbuffers[!horizontal as usize] } );  // bind texture of other framebuffer (or scene if first iteration)
                renderQuad(&mut quadVAO, &mut quadVBO);
                horizontal = !horizontal;
                if first_iteration {
                    first_iteration = false;
                }
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 3. now render floating point color buffer to 2D quad and tonemap HDR colors to default framebuffer's (clamped) color range
            // --------------------------------------------------------------------------------------------------------------------------
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            shaderBloomFinal.useProgram();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, colorBuffers[0]);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, pingpongColorbuffers[!horizontal as usize]);
            shaderBloomFinal.setInt(c_str!("bloom"), bloom as i32);
            shaderBloomFinal.setFloat(c_str!("exposure"), exposure);
            renderQuad(&mut quadVAO, &mut quadVBO);

            // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
            // -------------------------------------------------------------------------------
            window.swap_buffers();
            glfw.poll_events();
        }
    }
}

// renderCube() renders a 1x1 3D cube in NDC.
// -------------------------------------------------
unsafe fn renderCube(cubeVAO: &mut u32, cubeVBO: &mut u32) {
    if *cubeVAO == 0 {
        let vertices: [f32; 288] = [
            // back face
            -1.0, -1.0, -1.0,  0.0,  0.0, -1.0, 0.0, 0.0, // bottom-left
             1.0,  1.0, -1.0,  0.0,  0.0, -1.0, 1.0, 1.0, // top-right
             1.0, -1.0, -1.0,  0.0,  0.0, -1.0, 1.0, 0.0, // bottom-right
             1.0,  1.0, -1.0,  0.0,  0.0, -1.0, 1.0, 1.0, // top-right
            -1.0, -1.0, -1.0,  0.0,  0.0, -1.0, 0.0, 0.0, // bottom-left
            -1.0,  1.0, -1.0,  0.0,  0.0, -1.0, 0.0, 1.0, // top-left
            // front face
            -1.0, -1.0,  1.0,  0.0,  0.0,  1.0, 0.0, 0.0, // bottom-left
             1.0, -1.0,  1.0,  0.0,  0.0,  1.0, 1.0, 0.0, // bottom-right
             1.0,  1.0,  1.0,  0.0,  0.0,  1.0, 1.0, 1.0, // top-right
             1.0,  1.0,  1.0,  0.0,  0.0,  1.0, 1.0, 1.0, // top-right
            -1.0,  1.0,  1.0,  0.0,  0.0,  1.0, 0.0, 1.0, // top-left
            -1.0, -1.0,  1.0,  0.0,  0.0,  1.0, 0.0, 0.0, // bottom-left
            // left face
            -1.0,  1.0,  1.0, -1.0,  0.0,  0.0, 1.0, 0.0, // top-right
            -1.0,  1.0, -1.0, -1.0,  0.0,  0.0, 1.0, 1.0, // top-left
            -1.0, -1.0, -1.0, -1.0,  0.0,  0.0, 0.0, 1.0, // bottom-left
            -1.0, -1.0, -1.0, -1.0,  0.0,  0.0, 0.0, 1.0, // bottom-left
            -1.0, -1.0,  1.0, -1.0,  0.0,  0.0, 0.0, 0.0, // bottom-right
            -1.0,  1.0,  1.0, -1.0,  0.0,  0.0, 1.0, 0.0, // top-right
            // right face
             1.0,  1.0,  1.0,  1.0,  0.0,  0.0, 1.0, 0.0, // top-left
             1.0, -1.0, -1.0,  1.0,  0.0,  0.0, 0.0, 1.0, // bottom-right
             1.0,  1.0, -1.0,  1.0,  0.0,  0.0, 1.0, 1.0, // top-right
             1.0, -1.0, -1.0,  1.0,  0.0,  0.0, 0.0, 1.0, // bottom-right
             1.0,  1.0,  1.0,  1.0,  0.0,  0.0, 1.0, 0.0, // top-left
             1.0, -1.0,  1.0,  1.0,  0.0,  0.0, 0.0, 0.0, // bottom-left
            // bottom face
            -1.0, -1.0, -1.0,  0.0, -1.0,  0.0, 0.0, 1.0, // top-right
             1.0, -1.0, -1.0,  0.0, -1.0,  0.0, 1.0, 1.0, // top-left
             1.0, -1.0,  1.0,  0.0, -1.0,  0.0, 1.0, 0.0, // bottom-left
             1.0, -1.0,  1.0,  0.0, -1.0,  0.0, 1.0, 0.0, // bottom-left
            -1.0, -1.0,  1.0,  0.0, -1.0,  0.0, 0.0, 0.0, // bottom-right
            -1.0, -1.0, -1.0,  0.0, -1.0,  0.0, 0.0, 1.0, // top-right
            // top face
            -1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 0.0, 1.0, // top-left
             1.0,  1.0,  1.0,  0.0,  1.0,  0.0, 1.0, 0.0, // bottom-right
             1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 1.0, 1.0, // top-right
             1.0,  1.0,  1.0,  0.0,  1.0,  0.0, 1.0, 0.0, // bottom-right
            -1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 0.0, 1.0, // top-left
            -1.0,  1.0,  1.0,  0.0,  1.0,  0.0, 0.0, 0.0, // bottom-left
        ];
        gl::GenVertexArrays(1, cubeVAO);
        gl::GenBuffers(1, cubeVBO);
        // fill buffer
        gl::BindBuffer(gl::ARRAY_BUFFER, *cubeVBO);
        let size = (vertices.len() * mem::size_of::<f32>()) as isize;
        let data = &vertices[0] as *const f32 as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);
        // link vertex attributes
        gl::BindVertexArray(*cubeVAO);
        let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }
    // render Cube
    gl::BindVertexArray(*cubeVAO);
    gl::DrawArrays(gl::TRIANGLES, 0, 36);
    gl::BindVertexArray(0);
}

// renders a 1x1 quad in NDC with manually calculated tangent vectors
// ------------------------------------------------------------------
unsafe fn renderQuad(quadVAO: &mut u32, quadVBO: &mut u32) {
    if *quadVAO == 0 {
        let quadVertices: [f32; 20] = [
            // positions     // texture Coords
            -1.0,  1.0, 0.0, 0.0, 1.0,
            -1.0, -1.0, 0.0, 0.0, 0.0,
             1.0,  1.0, 0.0, 1.0, 1.0,
             1.0, -1.0, 0.0, 1.0, 0.0,
        ];

        // setup plane VAO
        gl::GenVertexArrays(1, quadVAO);
        gl::GenBuffers(1, quadVBO);
        gl::BindVertexArray(*quadVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, *quadVBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (quadVertices.len() * mem::size_of::<f32>()) as isize,
            &quadVertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW);
        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
    }
    gl::BindVertexArray(*quadVAO);
    gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    gl::BindVertexArray(0);
}

// NOTE: not the same version as in common.rs
pub fn processInput(
    window: &mut glfw::Window, deltaTime: f32, camera: &mut Camera,
    bloom: &mut bool, bloomKeyPressed: &mut bool, exposure: &mut f32) {
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

    if window.get_key(Key::Space) == Action::Press && !(*bloomKeyPressed) {
        *bloom = !(*bloom);
        *bloomKeyPressed = true;
        println!("bloom: {} | exposure: {}", if *bloom { "on" } else { "off" }, *exposure);
    }
    if window.get_key(Key::Space) == Action::Release {
        *bloomKeyPressed = false;
    }

    if window.get_key(Key::Q) == Action::Press {
        if *exposure > 0.0 {
            *exposure -= 0.01;
        }
        else {
            *exposure = 0.0;
        }
        println!("bloom: {} | exposure: {}", if *bloom { "on" } else { "off" }, *exposure);
    }
    if window.get_key(Key::E) == Action::Press {
        *exposure += 0.01;
        println!("bloom: {} | exposure: {}", if *bloom { "on" } else { "off" }, *exposure);
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
