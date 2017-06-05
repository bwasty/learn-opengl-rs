#![allow(non_upper_case_globals)]
extern crate glfw;
use self::glfw::{ Context, Key, Action };

extern crate gl;
use self::gl::types::*;

use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::os::raw::c_void;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const vertexShaderSource: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
       gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

const fragmentShaderSource: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;

const fragmentShader2Source: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
       FragColor = vec4(1.0f, 1.0f, 0.0f, 1.0f);
    }
"#;

#[allow(dead_code)]
#[allow(non_snake_case)]
fn main() {
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
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shaderProgramOrange, shaderProgramYellow, mut VBOs, mut VAOs) = unsafe {
        // build and compile our shader program
        // ------------------------------------
        // // we skipped compile log checks this time for readability (if you do encounter issues, add the compile-checks! see previous code samples)
        let vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
        let fragmentShaderOrange = gl::CreateShader(gl::FRAGMENT_SHADER); // the first fragment shader that outputs the color orange
        let fragmentShaderYellow = gl::CreateShader(gl::FRAGMENT_SHADER); // the second fragment shader that outputs the color yellow
        let shaderProgramOrange = gl::CreateProgram();
        let shaderProgramYellow = gl::CreateProgram(); // the second shader program
        let c_str_vert = CString::new(vertexShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(vertexShader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertexShader);
        let c_str_frag_orange = CString::new(fragmentShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(fragmentShaderOrange, 1, &c_str_frag_orange.as_ptr(), ptr::null());
        gl::CompileShader(fragmentShaderOrange);
        let c_str_frag_yellow = CString::new(fragmentShader2Source.as_bytes()).unwrap();
        gl::ShaderSource(fragmentShaderYellow, 1, &c_str_frag_yellow.as_ptr(), ptr::null());
        gl::CompileShader(fragmentShaderYellow);
        // link the first program object
        gl::AttachShader(shaderProgramOrange, vertexShader);
        gl::AttachShader(shaderProgramOrange, fragmentShaderOrange);
        gl::LinkProgram(shaderProgramOrange);
        // then link the second program object using a different fragment shader (but same vertex shader)
        // this is perfectly allowed since the inputs and outputs of both the vertex and fragment shaders are equally matched.
        gl::AttachShader(shaderProgramYellow, vertexShader);
        gl::AttachShader(shaderProgramYellow, fragmentShaderYellow);
        gl::LinkProgram(shaderProgramYellow);

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let firstTriangle: [f32; 9] = [
            -0.9, -0.5, 0.0,  // left
            -0.0, -0.5, 0.0,  // right
            -0.45, 0.5, 0.0,  // top
        ];
        let secondTriangle: [f32; 9] = [
            0.0, -0.5, 0.0,  // left
            0.9, -0.5, 0.0,  // right
            0.45, 0.5, 0.0   // top
        ];
        let (mut VBOs, mut VAOs) = ([0, 0], [0, 0]);
        gl::GenVertexArrays(2, VAOs.as_mut_ptr()); // we can also generate multiple VAOs or buffers at the same time
        gl::GenBuffers(2, VBOs.as_mut_ptr());
        // first triangle setup
        // --------------------
        gl::BindVertexArray(VAOs[0]);
        gl::BindBuffer(gl::ARRAY_BUFFER, VBOs[0]);
        // Vertex attributes stay the same
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (firstTriangle.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &firstTriangle[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);
        // gl::BindVertexArray(0); // no need to unbind at all as we directly bind a different VAO the next few lines
        // second triangle setup
        // ---------------------
        gl::BindVertexArray(VAOs[1]);
        gl::BindBuffer(gl::ARRAY_BUFFER, VBOs[1]);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (secondTriangle.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &secondTriangle[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null()); // because the vertex data is tightly packed we can also specify 0 as the vertex attribute's stride to let OpenGL figure it out
        gl::EnableVertexAttribArray(0);
        // gl::BindVertexArray(0); // not really necessary as well, but beware of calls that could affect VAOs while this one is bound (like binding element buffer objects, or enabling/disabling vertex attributes)

        // uncomment this call to draw in wireframe polygons.
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (shaderProgramOrange, shaderProgramYellow, VBOs, VAOs)
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
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // now when we draw the triangle we first use the vertex and orange fragment shader from the first program
            gl::UseProgram(shaderProgramOrange);
            // draw the first triangle using the data from our first VAO
            gl::BindVertexArray(VAOs[0]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);    // this call should output an orange triangle
            // then we draw the second triangle using the data from the second VAO
            // when we draw the second triangle we want to use a different shader program so we switch to the shader program with our yellow fragment shader.
            gl::UseProgram(shaderProgramYellow);
            gl::BindVertexArray(VAOs[1]);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    unsafe {
        gl::DeleteVertexArrays(2, VAOs.as_mut_ptr());
        gl::DeleteBuffers(2, VBOs.as_mut_ptr());
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            },
            _ => {},
        }
    }
}
