#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;
use self::gl::types::*;

use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::ffi::{CStr, CString};

use common::{process_events};
use shader::Shader;
use camera::Camera;
use camera::Camera_Movement::*;
use model::*;

use cgmath::*;
extern crate rand;
use self::rand::Rng;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + f * (b - a)
}

pub fn main_5_9() {
    // camera
    let mut camera = Camera {
        Position: Point3::new(0.0, 0.0, 5.0),
        ..Camera::default()
    };
    let mut lastX: f32 = SCR_WIDTH as f32 / 2.0;
    let mut lastY: f32 = SCR_HEIGHT as f32 / 2.0;
    let mut firstMouse = true;

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

    unsafe {  
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
    }

    // build and compile shaders
    // ------------------------------------
    let shaderGeometryPass = Shader::new(
        "src/_5_advanced_lighting/shaders/9.ssao_geometry.vs",
        "src/_5_advanced_lighting/shaders/9.ssao_geometry.fs"
    );
    let shaderLightingPass = Shader::new(
        "src/_5_advanced_lighting/shaders/9.ssao.vs",
        "src/_5_advanced_lighting/shaders/9.ssao_lighting.fs"
    );
    let shaderSSAO = Shader::new(
        "src/_5_advanced_lighting/shaders/9.ssao.vs",
        "src/_5_advanced_lighting/shaders/9.ssao.fs"
    );
    let shaderSSAOBlur = Shader::new(
        "src/_5_advanced_lighting/shaders/9.ssao.vs",
        "src/_5_advanced_lighting/shaders/9.ssao_blur.fs"
    );

    // load textures
    // -------------
    let backpack = Model::new("resources/objects/backpack/backpack.obj");

    let (gBuffer, gPosition, gNormal, gAlbedo, lightPos, lightColor, ssaoFBO, ssaoKernel, noiseTexture, ssaoBlurFBO, ssaoColorBuffer, ssaoColorBufferBlur) = unsafe {
        // configure floating point framebuffer
        // ------------------------------------
        let mut gBuffer = 0;
        gl::GenFramebuffers(1, &mut gBuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, gBuffer);

        let (mut gPosition, mut gNormal, mut gAlbedo) = (0, 0, 0);
        // position color buffer
        gl::GenTextures(1, &mut gPosition);
        gl::BindTexture(gl::TEXTURE_2D, gPosition);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as i32,
            SCR_WIDTH as i32, SCR_HEIGHT as i32, 0, gl::RGBA, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, gPosition, 0);
        // normal color buffer
        gl::GenTextures(1, &mut gNormal);
        gl::BindTexture(gl::TEXTURE_2D, gNormal);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as i32,
            SCR_WIDTH as i32, SCR_HEIGHT as i32, 0, gl::RGBA, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, gNormal, 0);
        // color + specular color buffer
        gl::GenTextures(1, &mut gAlbedo);
        gl::BindTexture(gl::TEXTURE_2D, gAlbedo);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA16F as i32,
            SCR_WIDTH as i32, SCR_HEIGHT as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, gAlbedo, 0);
        // tell OpenGL which color attachments we'll use (of this framebuffer) for rendering 
        let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2];
        gl::DrawBuffers(3, attachments.as_ptr());
        // create and attach depth buffer (renderbuffer)
        let mut rboDepth = 0;
        gl::GenRenderbuffers(1, &mut rboDepth);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rboDepth);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, SCR_WIDTH as i32, SCR_HEIGHT as i32);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rboDepth);
        // finally check if framebuffer is complete
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("Framebuffer not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // also create framebuffer to hold SSAO processing stage 
        // -----------------------------------------------------
        let (mut ssaoFBO, mut ssaoBlurFBO) = (0, 0);
        gl::GenFramebuffers(1, &mut ssaoFBO); gl::GenFramebuffers(1, &mut ssaoBlurFBO);
        gl::BindFramebuffer(gl::FRAMEBUFFER, ssaoFBO);
        let (mut ssaoColorBuffer, mut ssaoColorBufferBlur) = (0, 0);
        // SSAO color buffer
        gl::GenTextures(1, &mut ssaoColorBuffer);
        gl::BindTexture(gl::TEXTURE_2D, ssaoColorBuffer);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, SCR_WIDTH as i32, SCR_HEIGHT as i32, 0, gl::RED, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, ssaoColorBuffer, 0);
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("SSAO Framebuffer 2 not complete!");
        }
        // and blur stage
        gl::BindFramebuffer(gl::FRAMEBUFFER, ssaoBlurFBO);
        gl::GenTextures(1, &mut ssaoColorBufferBlur);
        gl::BindTexture(gl::TEXTURE_2D, ssaoColorBufferBlur);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, SCR_WIDTH as i32, SCR_HEIGHT as i32, 0, gl::RED, gl::FLOAT, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, ssaoColorBufferBlur, 0);
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("SSAO Blur Framebuffer 2 not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // generate sample kernel
        // ----------------------
        let mut ssaoKernel = Vec::new();
        let mut generator = rand::thread_rng();

        for i in 0 .. 64 {
            let mut sample = vec3(generator.gen_range(0.0, 1.0) * 2.0 - 1.0, generator.gen_range(0.0, 1.0) * 2.0 - 1.0, generator.gen_range(0.0, 1.0));
            sample = sample.normalize();
            sample *= generator.gen_range(0.0, 1.0);
            let mut scale = i as f32 / 64.0;

            // scale samples s.t. they're more aligned to center of kernel
            scale = lerp(0.1, 1.0, scale * scale);
            sample *= scale;
            ssaoKernel.push(sample);
        }

        // generate noise texture
        // ----------------------
        let mut ssaoNoise = Vec::new();
        for _ in 0 .. 16 {
            let noise = vec3(generator.gen_range(0.0, 1.0) * 2.0 - 1.0, generator.gen_range(0.0, 1.0) * 2.0 - 1.0, 0.0);
            ssaoNoise.push(noise);
        }
        let mut noiseTexture = 0; gl::GenTextures(1, &mut noiseTexture);
        gl::BindTexture(gl::TEXTURE_2D, noiseTexture);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA32F as i32, 4, 4, 0, gl::RGB, gl::FLOAT, ssaoNoise.as_ptr() as *const c_void);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

        // lighting info
        // -------------
        let lightPos = vec3(2.0, 4.0, -2.0);
        let lightColor = vec3(0.2, 0.2, 0.7);

        // shader configuration
        // --------------------
        shaderLightingPass.useProgram();
        shaderLightingPass.setInt(c_str!("gPosition"), 0);
        shaderLightingPass.setInt(c_str!("gNormal"), 1);
        shaderLightingPass.setInt(c_str!("gAlbedo"), 2);
        shaderLightingPass.setInt(c_str!("ssao"), 3);
        shaderSSAO.useProgram();
        shaderSSAO.setInt(c_str!("gPosition"), 0);
        shaderSSAO.setInt(c_str!("gNormal"), 1);
        shaderSSAO.setInt(c_str!("texNoise"), 2);
        shaderSSAOBlur.useProgram();
        shaderSSAOBlur.setInt(c_str!("ssaoInput"), 0);

        (gBuffer, gPosition, gNormal, gAlbedo, lightPos, lightColor, ssaoFBO, ssaoKernel, noiseTexture, ssaoBlurFBO, ssaoColorBuffer, ssaoColorBufferBlur)
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

        let mut cubeVAO = 0;
        let mut cubeVBO = 0;
        let mut quadVAO = 0;
        let mut quadVBO = 0;

        unsafe {
            // render
            // ------
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // 1. geometry pass: render scene's geometry/color data into gbuffer
            // -----------------------------------------------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, gBuffer);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 50.0);
                let view = camera.GetViewMatrix();
                shaderGeometryPass.useProgram();
                shaderGeometryPass.setMat4(c_str!("projection"), &projection);
                shaderGeometryPass.setMat4(c_str!("view"), &view);
                // room cube
                let mut model = Matrix4::<f32>::identity();
                model = model * Matrix4::<f32>::from_translation(vec3(0.0, 7.0, 0.0));
                model = model * Matrix4::<f32>::from_scale(7.5);
                shaderGeometryPass.setMat4(c_str!("model"), &model);
                shaderGeometryPass.setInt(c_str!("invertedNormals"), 1);
                renderCube(&mut cubeVAO, &mut cubeVBO);
                shaderGeometryPass.setInt(c_str!("invertedNormals"), 0);
                // backpack model on the floor
                model = Matrix4::identity();
                model = model * Matrix4::<f32>::from_translation(vec3(0.0, 0.5, 0.0));
                model = model * Matrix4::<f32>::from_axis_angle(vec3(1.0, 0.0, 0.0), Deg(-90.0));
                model = model * Matrix4::<f32>::from_scale(1.0);
                shaderGeometryPass.setMat4(c_str!("model"), &model);
                backpack.Draw(&shaderGeometryPass);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);


            // 2. generate SSAO texture
            // ------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, ssaoFBO);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                shaderSSAO.useProgram();
                // Send kernel + rotation 
                for i in 0 .. 64 {
                    let name = CString::new(format!("samples[{}]", i)).unwrap();
                    shaderSSAO.setVector3(&name, &ssaoKernel[i]);
                }
                shaderSSAO.setMat4(c_str!("projection"), &projection);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, gPosition);
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, gNormal);
                gl::ActiveTexture(gl::TEXTURE2);
                gl::BindTexture(gl::TEXTURE_2D, noiseTexture);
                renderQuad(&mut quadVAO, &mut quadVBO);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);


            // 3. blur SSAO texture to remove noise
            // ------------------------------------
            gl::BindFramebuffer(gl::FRAMEBUFFER, ssaoBlurFBO);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                shaderSSAOBlur.useProgram();
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, ssaoColorBuffer);
                renderQuad(&mut quadVAO, &mut quadVBO);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            

            // 4. lighting pass: traditional deferred Blinn-Phong lighting with added screen-space ambient occlusion
            // -----------------------------------------------------------------------------------------------------
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            shaderLightingPass.useProgram();
            // send light relevant uniforms
            let lightPos4 = vec4(lightPos.x, lightPos.y, lightPos.z, 1.0);
            let view = camera.GetViewMatrix() * lightPos4;
            let lightPosView = vec3(view.x, view.y, view.z);
            shaderLightingPass.setVec3(c_str!("light.Position"), lightPosView.x, lightPosView.y, lightPosView.z);
            shaderLightingPass.setVec3(c_str!("light.Color"), lightColor.x, lightColor.y, lightColor.z);
            // Update attenuation parameters
            let linear = 0.09;
            let quadratic = 0.032;

            shaderLightingPass.setVec3(c_str!("light.Linear"), linear, linear, linear);
            shaderLightingPass.setVec3(c_str!("light.Quadratic"), quadratic, quadratic, quadratic);
        
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gPosition);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, gNormal);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, gAlbedo);
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, ssaoColorBufferBlur);
            renderQuad(&mut quadVAO, &mut quadVBO);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
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
             1.0,  1.0, 1.0,  0.0,  1.0,  0.0, 1.0, 0.0, // bottom-right
             1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 1.0, 1.0, // top-right
             1.0,  1.0,  1.0,  0.0,  1.0,  0.0, 1.0, 0.0, // bottom-right
            -1.0,  1.0, -1.0,  0.0,  1.0,  0.0, 0.0, 1.0, // top-left
            -1.0,  1.0,  1.0,  0.0,  1.0,  0.0, 0.0, 0.0,  // bottom-left
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
    window: &mut glfw::Window, deltaTime: f32, camera: &mut Camera)
{
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
}
