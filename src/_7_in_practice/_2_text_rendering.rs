#![allow(non_upper_case_globals)]
extern crate glfw;
use self::glfw::{Action, Context, Key};

extern crate gl;
use self::gl::types::*;

use std::collections::HashMap;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::sync::mpsc::Receiver;

use cgmath::{Matrix, Matrix4};

use shader::Shader;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

struct Character {
    texture: u32,
    size: cgmath::Vector2<i32>,
    bearing: cgmath::Vector2<i32>,
    advance: u32,
}

#[allow(non_snake_case)]
pub fn main_7_2() {
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader, vao, vbo, characters) = unsafe {
        // OpenGL state
        // ------------
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        // compile and setup the shader
        // ----------------------------
        let shader = Shader::new("src/_7_in_practice/shaders/text.vs", "src/_7_in_practice/shaders/text.fs");
        let projection: Matrix4<f32> = cgmath::ortho(0.0, SCR_WIDTH as f32, 0.0, SCR_HEIGHT as f32, -1.0, 1.0);
        shader.useProgram();
        let projection_loc = gl::GetUniformLocation(shader.ID, c_str!("projection").as_ptr());
        gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection.as_ptr());

        // Init the library
        let lib = freetype::Library::init().expect("ERROR::FREETYPE: Could not init FreeType Library");
        // find path to font
        let font_name = Path::new("resources/fonts/Antonio-Bold.ttf");
        if !font_name.exists() {
            print!("ERROR::FREETYPE: Failed to load font_name");
            return;
        }
        // Load a font face
        let face = lib
            .new_face(font_name, 0)
            .expect("ERROR::FREETYPE: Failed to load font");

        // set size to load glyphs as
        face.set_pixel_sizes(0, 48).unwrap();

        // disable byte-alignment restriction
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

        let mut characters: HashMap<GLchar, Character> = HashMap::new();

        // load first 128 characters of ASCII set
        for c in 0..128 {
            // Load character glyph
            face.load_char(c, freetype::face::LoadFlag::RENDER)
                .expect("ERROR::FREETYTPE: Failed to load Glyph");

            let bitmap = face.glyph().bitmap();
            // generate texture
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                bitmap.width(),
                bitmap.rows() as i32,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                bitmap.buffer().as_ptr() as *const c_void,
            );
            // set texture options
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            // now store character for later use
            let character = Character {
                texture,
                size: cgmath::vec2(bitmap.width(), bitmap.rows()),
                bearing: cgmath::vec2(face.glyph().bitmap_left(), face.glyph().bitmap_top()),
                advance: face.glyph().advance().x as u32,
            };

            characters.insert(c as GLchar, character);
        }
        gl::BindTexture(gl::TEXTURE_2D, 0);

        // configure VAO/VBO for texture quads
        // -----------------------------------
        let (mut VBO, mut VAO) = (0, 0);
        gl::GenVertexArrays(1, &mut VAO);
        gl::GenBuffers(1, &mut VBO);
        gl::BindVertexArray(VAO);
        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(gl::ARRAY_BUFFER, 6 * 4 * mem::size_of::<f32>() as GLsizeiptr, ptr::null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * mem::size_of::<f32>() as GLsizei, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        (shader, VAO, VBO, characters)
    };

    // render loop
    // -----------
    while !window.should_close() {
        // events
        // -----

        process_events(&mut window, &events);
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            render_text(
                &shader,
                vao,
                vbo,
                &characters,
                String::from("This is sample text"),
                25.0,
                25.0,
                1.0,
                cgmath::vec3(0.5, 0.8, 0.2),
            );
            render_text(
                &shader,
                vao,
                vbo,
                &characters,
                String::from("(C) LearnOpenGL.com"),
                540.0,
                570.0,
                0.5,
                cgmath::vec3(0.3, 0.7, 0.9),
            );
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
}

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

// render line of text
// -------------------
unsafe fn render_text(
    shader: &Shader,
    vao: u32,
    vbo: u32,
    characters: &HashMap<GLchar, Character>,
    text: String,
    x: f32,
    y: f32,
    scale: f32,
    color: cgmath::Vector3<f32>,
) {
    // activate corresponding render state
    shader.useProgram();
    let text_color_loc = gl::GetUniformLocation(shader.ID, c_str!("textColor").as_ptr());
    gl::Uniform3f(text_color_loc, color.x, color.y, color.z);
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindVertexArray(vao);

    let mut x_pos = x;

    // iterate through all characters
    for c in text.chars() {
        let key_c = c as GLchar;
        let ch = &characters[&key_c];

        let xpos = x_pos + ch.bearing.x as f32 * scale;
        let ypos = y - (ch.size.y - ch.bearing.y) as f32 * scale;

        let w = ch.size.x as f32 * scale;
        let h = ch.size.y as f32 * scale;

        #[rustfmt::skip]
        let vertices: [[f32; 4]; 6] = [
            [ xpos,     ypos + h,   0.0, 0.0 ],
            [ xpos,     ypos,       0.0, 1.0 ],
            [ xpos + w, ypos,       1.0, 1.0 ],

            [ xpos,     ypos + h,   0.0, 0.0 ],
            [ xpos + w, ypos,       1.0, 1.0 ],
            [ xpos + w, ypos + h,   1.0, 0.0 ],
        ];

        // render glyph texture over quad
        gl::BindTexture(gl::TEXTURE_2D, ch.texture);
        // update content of VBO memory
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferSubData(gl::ARRAY_BUFFER, 0, mem::size_of_val(&vertices) as GLsizeiptr, mem::transmute(&vertices[0])); // be sure to use gl::BufferSubData and not gl::BufferData

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        // render quad
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
        // now advance cursors for next glyph (note that advance is number of 1/64 pixels)
        x_pos += (ch.advance >> 6) as f32 * scale; // bitshift by 6 to get value in pixels (2^6 = 64 (divide amount of 1/64th pixels by 64 to get amount of pixels))
    }

    gl::BindVertexArray(0);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}
