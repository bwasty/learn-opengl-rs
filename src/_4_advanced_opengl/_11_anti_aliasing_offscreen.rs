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

use common::{process_events, processInput};
use shader::Shader;
use camera::Camera;

use cgmath::{Matrix4,  Deg, perspective, Point3};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 1280;
const SCR_HEIGHT: u32 = 720;

pub fn main_4_11() {
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

    // query framebuffer size as it might be quite different from the requested size on Retina displays
    let (scr_width, scr_height) = window.get_framebuffer_size();

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);

    // tell GLFW to capture our mouse
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader, screenShader, cubeVAO, quadVAO, framebuffer, intermediateFBO, screenTexture) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile our shader program
        // ------------------------------------
        let shader = Shader::new(
            "src/_4_advanced_opengl/shaders/11.anti_aliasing.vs",
            "src/_4_advanced_opengl/shaders/11.anti_aliasing.fs");
        let screenShader = Shader::new(
            "src/_4_advanced_opengl/shaders/11.aa_post.vs",
            "src/_4_advanced_opengl/shaders/11.aa_post.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let cubeVertices: [f32; 108] = [
             // positions
             -0.5, -0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5,  0.5, -0.5,
              0.5,  0.5, -0.5,
             -0.5,  0.5, -0.5,
             -0.5, -0.5, -0.5,

             -0.5, -0.5,  0.5,
              0.5, -0.5,  0.5,
              0.5,  0.5,  0.5,
              0.5,  0.5,  0.5,
             -0.5,  0.5,  0.5,
             -0.5, -0.5,  0.5,

             -0.5,  0.5,  0.5,
             -0.5,  0.5, -0.5,
             -0.5, -0.5, -0.5,
             -0.5, -0.5, -0.5,
             -0.5, -0.5,  0.5,
             -0.5,  0.5,  0.5,

              0.5,  0.5,  0.5,
              0.5,  0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5, -0.5,  0.5,
              0.5,  0.5,  0.5,

             -0.5, -0.5, -0.5,
              0.5, -0.5, -0.5,
              0.5, -0.5,  0.5,
              0.5, -0.5,  0.5,
             -0.5, -0.5,  0.5,
             -0.5, -0.5, -0.5,

             -0.5,  0.5, -0.5,
              0.5,  0.5, -0.5,
              0.5,  0.5,  0.5,
              0.5,  0.5,  0.5,
             -0.5,  0.5,  0.5,
             -0.5,  0.5, -0.5,
        ];
        let quadVertices: [f32; 24] = [ // vertex attributes for a quad that fills the entire screen in Normalized Device Coordinates.
            // positions // texCoords
            -1.0,  1.0,  0.0, 1.0,
            -1.0, -1.0,  0.0, 0.0,
             1.0, -1.0,  1.0, 0.0,

            -1.0,  1.0,  0.0, 1.0,
             1.0, -1.0,  1.0, 0.0,
             1.0,  1.0,  1.0, 1.0
        ];
        // setup cube VAO
        let (mut cubeVAO, mut cubeVBO) = (0, 0);
        gl::GenVertexArrays(1, &mut cubeVAO);
        gl::GenBuffers(1, &mut cubeVBO);
        gl::BindVertexArray(cubeVAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, cubeVBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (cubeVertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &cubeVertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        let stride = 3 * mem::size_of::<GLfloat>() as GLsizei;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        // setup screen VAO
        let (mut quadVAO, mut quadVBO) = (0, 0);
        gl::GenVertexArrays(1, &mut quadVAO);
        gl::GenBuffers(1, &mut quadVBO);
        gl::BindVertexArray(quadVBO);
        gl::BindBuffer(gl::ARRAY_BUFFER, quadVBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (quadVertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &quadVertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        let stride = 4 * mem::size_of::<GLfloat>() as GLsizei;
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<GLfloat>()) as *const c_void);

        // configure MSAA framebuffer
        // -------------------------
        let mut framebuffer = 0;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        // create a multisampled color attachment texture
        let mut textureColorBufferMultiSampled = 0;
        gl::GenTextures(1, &mut textureColorBufferMultiSampled);
        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, textureColorBufferMultiSampled);
        gl::TexImage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE, 4, gl::RGB, scr_width, scr_height, gl::TRUE);
        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D_MULTISAMPLE, textureColorBufferMultiSampled, 0);
        // create a (also multisampled) renderbuffer object for depth and stencil attachments
        let mut rbo = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::DEPTH24_STENCIL8, scr_width, scr_height);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        // configure second post-processing framebuffer
        let mut intermediateFBO = 0;
        gl::GenFramebuffers(1, &mut intermediateFBO);
        gl::BindFramebuffer(gl::FRAMEBUFFER, intermediateFBO);

        // create a color attachment texture
        let mut screenTexture = 0;
        gl::GenTextures(1, &mut screenTexture);
        gl::BindTexture(gl::TEXTURE_2D, screenTexture);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, scr_width, scr_height, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, screenTexture, 0);	// we only need a color buffer

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            println!("ERROR::FRAMEBUFFER:: Intermediate framebuffer is not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        screenShader.useProgram();
        screenShader.setInt(c_str!("screenTexture"), 0);

        (shader, screenShader, cubeVAO, quadVAO, framebuffer, intermediateFBO, screenTexture)
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

        // render loop
        // ------
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // 1. draw scene as normal in multisampled buffers
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // set transformation matrices
            shader.useProgram();
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32 , 0.1, 100.0);
            shader.setMat4(c_str!("projection"), &projection);
            shader.setMat4(c_str!("view"), &camera.GetViewMatrix());
            shader.setMat4(c_str!("model"), &Matrix4::identity());

            gl::BindVertexArray(cubeVAO);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            // 2. now blit multisampled buffer(s) to normal colorbuffer of intermediate FBO. Image is stored in screenTexture
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, intermediateFBO);
            gl::BlitFramebuffer(0, 0, scr_width, scr_height, 0, 0, scr_width, scr_height, gl::COLOR_BUFFER_BIT, gl::NEAREST);

            // 3. now render quad with scene's visuals as its texture image
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::DEPTH_TEST);

            // draw Screen quad
            screenShader.useProgram();
            gl::BindVertexArray(quadVAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, screenTexture);	// use the now resolved color attachment as the quad's texture
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}
