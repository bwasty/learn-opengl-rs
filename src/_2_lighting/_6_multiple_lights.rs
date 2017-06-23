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
use std::ffi::CStr;
use std::path::Path;

use shader::Shader;
use camera::Camera;
use camera::Camera_Movement::*;

use cgmath::{Matrix4, Vector3, vec3, Point3, Deg, perspective};
use cgmath::prelude::*;

use image;
use image::GenericImage;
use image::DynamicImage::*;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

pub fn main_2_6() {
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

    let (lightingShader, lampShader, VBO, cubeVAO, lightVAO, diffuseMap, specularMap, cubePositions, pointLightPositions) = unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);

        // build and compile our shader program
        // ------------------------------------
        let lightingShader = Shader::new("src/_2_lighting/shaders/6.multiple_lights.vs", "src/_2_lighting/shaders/6.multiple_lights.fs");
        let lampShader = Shader::new("src/_2_lighting/shaders/6.lamp.vs", "src/_2_lighting/shaders/6.lamp.fs");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        let vertices: [f32; 288] = [
            // positions       // normals        // texture coords
            -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0,  0.0,
             0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  1.0,  0.0,
             0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0,  1.0,
             0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0,  1.0,
            -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  0.0,  1.0,
            -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0,  0.0,

            -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  0.0,  0.0,
             0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  1.0,  0.0,
             0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  1.0,  1.0,
             0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  1.0,  1.0,
            -0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  0.0,  1.0,
            -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  0.0,  0.0,

            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0,  0.0,
            -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,  1.0,  1.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0,  1.0,
            -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0,  1.0,
            -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,  0.0,  0.0,
            -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0,  0.0,

             0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0,  0.0,
             0.5,  0.5, -0.5,  1.0,  0.0,  0.0,  1.0,  1.0,
             0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0,  1.0,
             0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  0.0,  1.0,
             0.5, -0.5,  0.5,  1.0,  0.0,  0.0,  0.0,  0.0,
             0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  1.0,  0.0,

            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0,  1.0,
             0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  1.0,  1.0,
             0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0,  0.0,
             0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0,  0.0,
            -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  0.0,  0.0,
            -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0,  1.0,

            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0,  1.0,
             0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  1.0,  1.0,
             0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0,  0.0,
             0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0,  0.0,
            -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  0.0,  0.0,
            -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0,  1.0
        ];
        // positions all containers
        let cubePositions: [Vector3<f32>; 10] = [
            vec3( 0.0,  0.0,  0.0),
            vec3( 2.0,  5.0, -15.0),
            vec3(-1.5, -2.2, -2.5),
            vec3(-3.8, -2.0, -12.3),
            vec3( 2.4, -0.4, -3.5),
            vec3(-1.7,  3.0, -7.5),
            vec3( 1.3, -2.0, -2.5),
            vec3( 1.5,  2.0, -2.5),
            vec3( 1.5,  0.2, -1.5),
            vec3(-1.3,  1.0, -1.5)
        ];
        // positions of the point lights
        let pointLightPositions: [Vector3<f32>; 4] = [
            vec3( 0.7,  0.2,  2.0),
            vec3( 2.3, -3.3, -4.0),
            vec3(-4.0,  2.0, -12.0),
            vec3( 0.0,  0.0, -3.0)
        ];
        // first, configure the cube's VAO (and VBO)
        let (mut VBO, mut cubeVAO) = (0, 0);
        gl::GenVertexArrays(1, &mut cubeVAO);
        gl::GenBuffers(1, &mut VBO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::BindVertexArray(cubeVAO);
        let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(2);

        // second, configure the light's VAO (VBO stays the same; the vertices are the same for the light object which is also a 3D cube)
        let mut lightVAO = 0;
        gl::GenVertexArrays(1, &mut lightVAO);
        gl::BindVertexArray(lightVAO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        // note that we update the lamp's position attribute's stride to reflect the updated buffer data
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // load textures (we now use a utility function to keep the code more organized)
        // -----------------------------------------------------------------------------
        let diffuseMap = loadTexture("resources/textures/container2.png");
        let specularMap = loadTexture("resources/textures/container2_specular.png");

        // shader configuration
        // --------------------
        lightingShader.useProgram();
        lightingShader.setInt(c_str!("material.diffuse"), 0);
        lightingShader.setInt(c_str!("material.specular"), 1);

        (lightingShader, lampShader, VBO, cubeVAO, lightVAO, diffuseMap, specularMap, cubePositions, pointLightPositions)
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


        // render
        // ------
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // be sure to activate shader when setting uniforms/drawing objects
            lightingShader.useProgram();
            lightingShader.setVector3(c_str!("viewPos"), &camera.Position.to_vec());
            lightingShader.setFloat(c_str!("material.shininess"), 32.0);
            /*
                Here we set all the uniforms for the 5/6 types of lights we have. We have to set them manually and index
                the proper PointLight struct in the array to set each uniform variable. This can be done more code-friendly
                by defining light types as classes and set their values in there, or by using a more efficient uniform approach
                by using 'Uniform buffer objects', but that is something we'll discuss in the 'Advanced GLSL' tutorial.
            */
            // directional light
            lightingShader.setVec3(c_str!("dirLight.direction"), -0.2, -1.0, -0.3);
            lightingShader.setVec3(c_str!("dirLight.ambient"), 0.05, 0.05, 0.05);
            lightingShader.setVec3(c_str!("dirLight.diffuse"), 0.4, 0.4, 0.4);
            lightingShader.setVec3(c_str!("dirLight.specular"), 0.5, 0.5, 0.5);
            // point light 1
            lightingShader.setVector3(c_str!("pointLights[0].position"), &pointLightPositions[0]);
            lightingShader.setVec3(c_str!("pointLights[0].ambient"), 0.05, 0.05, 0.05);
            lightingShader.setVec3(c_str!("pointLights[0].diffuse"), 0.8, 0.8, 0.8);
            lightingShader.setVec3(c_str!("pointLights[0].specular"), 1.0, 1.0, 1.0);
            lightingShader.setFloat(c_str!("pointLights[0].constant"), 1.0);
            lightingShader.setFloat(c_str!("pointLights[0].linear"), 0.09);
            lightingShader.setFloat(c_str!("pointLights[0].quadratic"), 0.032);
            // point light 2
            lightingShader.setVector3(c_str!("pointLights[1].position"), &pointLightPositions[1]);
            lightingShader.setVec3(c_str!("pointLights[1].ambient"), 0.05, 0.05, 0.05);
            lightingShader.setVec3(c_str!("pointLights[1].diffuse"), 0.8, 0.8, 0.8);
            lightingShader.setVec3(c_str!("pointLights[1].specular"), 1.0, 1.0, 1.0);
            lightingShader.setFloat(c_str!("pointLights[1].constant"), 1.0);
            lightingShader.setFloat(c_str!("pointLights[1].linear"), 0.09);
            lightingShader.setFloat(c_str!("pointLights[1].quadratic"), 0.032);
            // point light 3
            lightingShader.setVector3(c_str!("pointLights[2].position"), &pointLightPositions[2]);
            lightingShader.setVec3(c_str!("pointLights[2].ambient"), 0.05, 0.05, 0.05);
            lightingShader.setVec3(c_str!("pointLights[2].diffuse"), 0.8, 0.8, 0.8);
            lightingShader.setVec3(c_str!("pointLights[2].specular"), 1.0, 1.0, 1.0);
            lightingShader.setFloat(c_str!("pointLights[2].constant"), 1.0);
            lightingShader.setFloat(c_str!("pointLights[2].linear"), 0.09);
            lightingShader.setFloat(c_str!("pointLights[2].quadratic"), 0.032);
            // point light 4
            lightingShader.setVector3(c_str!("pointLights[3].position"), &pointLightPositions[3]);
            lightingShader.setVec3(c_str!("pointLights[3].ambient"), 0.05, 0.05, 0.05);
            lightingShader.setVec3(c_str!("pointLights[3].diffuse"), 0.8, 0.8, 0.8);
            lightingShader.setVec3(c_str!("pointLights[3].specular"), 1.0, 1.0, 1.0);
            lightingShader.setFloat(c_str!("pointLights[3].constant"), 1.0);
            lightingShader.setFloat(c_str!("pointLights[3].linear"), 0.09);
            lightingShader.setFloat(c_str!("pointLights[3].quadratic"), 0.032);
            // spotLight
            lightingShader.setVector3(c_str!("spotLight.position"), &camera.Position.to_vec());
            lightingShader.setVector3(c_str!("spotLight.direction"), &camera.Front);
            lightingShader.setVec3(c_str!("spotLight.ambient"), 0.0, 0.0, 0.0);
            lightingShader.setVec3(c_str!("spotLight.diffuse"), 1.0, 1.0, 1.0);
            lightingShader.setVec3(c_str!("spotLight.specular"), 1.0, 1.0, 1.0);
            lightingShader.setFloat(c_str!("spotLight.constant"), 1.0);
            lightingShader.setFloat(c_str!("spotLight.linear"), 0.09);
            lightingShader.setFloat(c_str!("spotLight.quadratic"), 0.032);
            lightingShader.setFloat(c_str!("spotLight.cutOff"), 12.5f32.to_radians().cos());
            lightingShader.setFloat(c_str!("spotLight.outerCutOff"), 15.0f32.to_radians().cos());

            // view/projection transformations
            let projection: Matrix4<f32> = perspective(Deg(camera.Zoom), SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, 100.0);
            let view = camera.GetViewMatrix();
            lightingShader.setMat4(c_str!("projection"), &projection);
            lightingShader.setMat4(c_str!("view"), &view);

            // world transformation
            let mut model = Matrix4::<f32>::identity();
            lightingShader.setMat4(c_str!("model"), &model);

            // bind diffuse map
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, diffuseMap);
            // bind specular map
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, specularMap);

            // render containers
            gl::BindVertexArray(cubeVAO);
            for (i, position) in cubePositions.iter().enumerate() {
                // calculate the model matrix for each object and pass it to shader before drawing
                let mut model: Matrix4<f32> = Matrix4::from_translation(*position);
                let angle = 20.0 * i as f32;
                // don't forget to normalize the axis!
                model = model * Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg(angle));
                lightingShader.setMat4(c_str!("model"), &model);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

            // also draw the lamp object(s)
            lampShader.useProgram();
            lampShader.setMat4(c_str!("projection"), &projection);
            lampShader.setMat4(c_str!("view"), &view);

            // we now draw as many light bulbs as we have point lights.
            gl::BindVertexArray(lightVAO);
            for position in &pointLightPositions {
                model = Matrix4::from_translation(*position);
                model = model * Matrix4::from_scale(0.2); // Make it a smaller cube
                lampShader.setMat4(c_str!("model"), &model);

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
        gl::DeleteVertexArrays(1, &cubeVAO);
        gl::DeleteVertexArrays(1, &lightVAO);
        gl::DeleteBuffers(1, &VBO);
    }
}

fn process_events(events: &Receiver<(f64, glfw::WindowEvent)>,
                  firstMouse: &mut bool,
                  lastX: &mut f32,
                  lastY: &mut f32,
                  camera: &mut Camera) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::CursorPos(xpos, ypos) => {
                let (xpos, ypos) = (xpos as f32, ypos as f32);
                if *firstMouse {
                    *lastX = xpos;
                    *lastY = ypos;
                    *firstMouse = false;
                }

                let xoffset = xpos - *lastX;
                let yoffset = *lastY - ypos; // reversed since y-coordinates go from bottom to top

                *lastX = xpos;
                *lastY = ypos;

                camera.ProcessMouseMovement(xoffset, yoffset, true);
            }
            glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
                camera.ProcessMouseScroll(yoffset as f32);
            }
            _ => {}
        }
    }
}

fn processInput(window: &mut glfw::Window, deltaTime: f32, camera: &mut Camera) {
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

unsafe fn loadTexture(path: &str) -> u32 {
    let mut textureID = 0;

    gl::GenTextures(1, &mut textureID);
    let img = image::open(&Path::new(path)).expect("Texture failed to load");
    let format = match img {
        ImageLuma8(_) => gl::RED,
        ImageLumaA8(_) => gl::RG,
        ImageRgb8(_) => gl::RGB,
        ImageRgba8(_) => gl::RGBA,
    };

    let data = img.raw_pixels();

    gl::BindTexture(gl::TEXTURE_2D, textureID);
    gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, img.width() as i32, img.height() as i32,
        0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);
    gl::GenerateMipmap(gl::TEXTURE_2D);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    textureID
}
