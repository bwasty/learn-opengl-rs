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
use image::GenericImage;

use cgmath::{Matrix4, vec3,  Deg, Rad, perspective};
use cgmath::prelude::*;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

unsafe fn glCheckError_(file: &str, line: u32) -> u32 {
    let mut errorCode = gl::GetError();
    while errorCode != gl::NO_ERROR {
        let error = match errorCode {
            gl::INVALID_ENUM => "INVALID_ENUM",
            gl::INVALID_VALUE => "INVALID_VALUE",
            gl::INVALID_OPERATION => "INVALID_OPERATION",
            gl::STACK_OVERFLOW => "STACK_OVERFLOW",
            gl::STACK_UNDERFLOW => "STACK_UNDERFLOW",
            gl::OUT_OF_MEMORY => "OUT_OF_MEMORY",
            gl::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
            _ => "unknown GL error code"
        };

        println!("{} | {} ({})", error, file, line);

        errorCode = gl::GetError();
    }
    errorCode
}

macro_rules! glCheckError {
    () => (
        glCheckError_(file!(), line!())
    )
}

extern "system" fn glDebugOutput(source: gl::types::GLenum,
                                 type_: gl::types::GLenum,
                                 id: gl::types::GLuint,
                                 severity: gl::types::GLenum,
                                 _length: gl::types::GLsizei,
                                 message: *const gl::types::GLchar,
                                 _userParam: *mut c_void)
{
    if id == 131169 || id == 131185 || id == 131218 || id == 131204 {
        // ignore these non-significant error codes
        return
    }

    println!("---------------");
    let message = unsafe { CStr::from_ptr(message).to_str().unwrap() };
    println!("Debug message ({}): {}", id, message);
    match source {
        gl::DEBUG_SOURCE_API =>             println!("Source: API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM =>   println!("Source: Window System"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => println!("Source: Shader Compiler"),
        gl::DEBUG_SOURCE_THIRD_PARTY =>     println!("Source: Third Party"),
        gl::DEBUG_SOURCE_APPLICATION =>     println!("Source: Application"),
        gl::DEBUG_SOURCE_OTHER =>           println!("Source: Other"),
        _ =>                                println!("Source: Unknown enum value")
    }

    match type_ {
       gl::DEBUG_TYPE_ERROR =>               println!("Type: Error"),
       gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("Type: Deprecated Behaviour"),
       gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR =>  println!("Type: Undefined Behaviour"),
       gl::DEBUG_TYPE_PORTABILITY =>         println!("Type: Portability"),
       gl::DEBUG_TYPE_PERFORMANCE =>         println!("Type: Performance"),
       gl::DEBUG_TYPE_MARKER =>              println!("Type: Marker"),
       gl::DEBUG_TYPE_PUSH_GROUP =>          println!("Type: Push Group"),
       gl::DEBUG_TYPE_POP_GROUP =>           println!("Type: Pop Group"),
       gl::DEBUG_TYPE_OTHER =>               println!("Type: Other"),
       _ =>                                  println!("Type: Unknown enum value")
    }

    match severity {
       gl::DEBUG_SEVERITY_HIGH =>         println!("Severity: high"),
       gl::DEBUG_SEVERITY_MEDIUM =>       println!("Severity: medium"),
       gl::DEBUG_SEVERITY_LOW =>          println!("Severity: low"),
       gl::DEBUG_SEVERITY_NOTIFICATION => println!("Severity: notification"),
       _ =>                               println!("Severity: Unknown enum value")
    }
}

#[allow(non_snake_case)]
pub fn main_7_1() {
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true)); // comment this line in a release build!
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // tell GLFW to capture our mouse
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader, cubeVAO, texture) = unsafe {
        // enable OpenGL debug context if context allows for debug context
        let mut flags = 0;
        gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags);
        if flags as u32 & gl::CONTEXT_FLAG_DEBUG_BIT != 0 {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS); // makes sure errors are displayed synchronously
            gl::DebugMessageCallback(glDebugOutput, ptr::null());
            gl::DebugMessageControl(gl::DONT_CARE, gl::DONT_CARE, gl::DONT_CARE, 0, ptr::null(), gl::TRUE);
        }
        else {
            println!("Debug Context not active! Check if your driver supports the extension.")
        }

        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);

        // OpenGL initial state
        let shader = Shader::new("src/_7_in_practice/shaders/debugging.vs", "src/_7_in_practice/shaders/debugging.fs");

        // configure 3D cube
        let (mut cubeVAO, mut cubeVBO) = (0, 0);
        let vertices: [f32; 180] = [
            // back face
            -0.5, -0.5, -0.5,  0.0,  0.0, // Bottom-left
             0.5,  0.5, -0.5,  1.0,  1.0, // top-right
             0.5, -0.5, -0.5,  1.0,  0.0, // bottom-right
             0.5,  0.5, -0.5,  1.0,  1.0, // top-right
            -0.5, -0.5, -0.5,  0.0,  0.0, // bottom-left
            -0.5,  0.5, -0.5,  0.0,  1.0, // top-left
            // front face
            -0.5, -0.5,  0.5,  0.0,  0.0, // bottom-left
             0.5, -0.5,  0.5,  1.0,  0.0, // bottom-right
             0.5,  0.5,  0.5,  1.0,  1.0, // top-right
             0.5,  0.5,  0.5,  1.0,  1.0, // top-right
            -0.5,  0.5,  0.5,  0.0,  1.0, // top-left
            -0.5, -0.5,  0.5,  0.0,  0.0, // bottom-left
            // left face
            -0.5,  0.5,  0.5, -1.0,  0.0, // top-right
            -0.5,  0.5, -0.5, -1.0,  1.0, // top-left
            -0.5, -0.5, -0.5, -0.0,  1.0, // bottom-left
            -0.5, -0.5, -0.5, -0.0,  1.0, // bottom-left
            -0.5, -0.5,  0.5, -0.0,  0.0, // bottom-right
            -0.5,  0.5,  0.5, -1.0,  0.0, // top-right
            // right face
             0.5,  0.5,  0.5,  1.0,  0.0, // top-left
             0.5, -0.5, -0.5,  0.0,  1.0, // bottom-right
             0.5,  0.5, -0.5,  1.0,  1.0, // top-right
             0.5, -0.5, -0.5,  0.0,  1.0, // bottom-right
             0.5,  0.5,  0.5,  1.0,  0.0, // top-left
             0.5, -0.5,  0.5,  0.0,  0.0, // bottom-left
            // bottom face
            -0.5, -0.5, -0.5,  0.0,  1.0, // top-right
             0.5, -0.5, -0.5,  1.0,  1.0, // top-left
             0.5, -0.5,  0.5,  1.0,  0.0, // bottom-left
             0.5, -0.5,  0.5,  1.0,  0.0, // bottom-left
            -0.5, -0.5,  0.5,  0.0,  0.0, // bottom-right
            -0.5, -0.5, -0.5,  0.0,  1.0, // top-right
            // top face
            -0.5,  0.5, -0.5,  0.0,  1.0, // top-left
             0.5,  0.5,  0.5,  1.0,  0.0, // bottom-right
             0.5,  0.5, -0.5,  1.0,  1.0, // top-right
             0.5,  0.5,  0.5,  1.0,  0.0, // bottom-right
            -0.5,  0.5, -0.5,  0.0,  1.0, // top-left
            -0.5,  0.5,  0.5,  0.0,  0.0  // bottom-left
        ];
        gl::GenVertexArrays(1, &mut cubeVAO);
        gl::GenBuffers(1, &mut cubeVBO);
        // fill buffer
        gl::BindBuffer(gl::ARRAY_BUFFER, cubeVBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);
        // link vertex attributes
        gl::BindVertexArray(cubeVAO);
        gl::EnableVertexAttribArray(0);
        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        // load cube texture
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        let img = image::open(&Path::new("resources/textures/wood.png")).expect("Failed to load texture");
        let data = img.raw_pixels();
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       gl::RGB as i32,
                       img.width() as i32,
                       img.height() as i32,
                       0,
                       gl::RGB,
                       gl::UNSIGNED_BYTE,
                       &data[0] as *const u8 as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        // set up projection matrix
        let projection: Matrix4<f32> = perspective(Deg(45.0), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
        shader.setMat4(c_str!("projection"), &projection);
        shader.setInt(c_str!("tex"), 0);

        glCheckError!();

        (shader, cubeVAO, texture)
    };

    // render loop
    // -----------
    while !window.should_close() {
        // events
        // -----
        process_events(&mut window, &events);

        // render
        // ------
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            shader.useProgram();
            let rotationSpeed = 10.0;
            let angle = glfw.get_time() as f32 * rotationSpeed;
            let mut model: Matrix4<f32> = Matrix4::from_translation(vec3(0., 0., -2.5));
            model = model * Matrix4::from_axis_angle(vec3(1.0, 1.0, 1.0).normalize(), Rad(angle));
            shader.setMat4(c_str!("model"), &model);

            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::BindVertexArray(cubeVAO);
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::BindVertexArray(0);

        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

// NOTE: not the same version as in common.rs!
fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => {}
        }
    }
}
