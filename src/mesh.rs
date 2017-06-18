#![allow(non_snake_case)]
#![allow(dead_code)] // TODO!! TMP

use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

use cgmath::{ Vector3, Vector2 };
use gl;
use gl::types::*;

// NOTE: Always use repr(C) for structs passed to OpenGL!
// Otherwise the compiler may reorder the fields
#[repr(C)]
struct Vertex {
    // position
    Position: Vector3<f32>,
    // normal
    Normal: Vector3<f32>,
    // texCoords
    TexCoords: Vector2<f32>,
    // tangent
    Tangent: Vector3<f32>,
    // bitangent
    Bitangent: Vector3<f32>,
}

struct Texture {
    id: u32,
    type_: String,
    path: String,
}

struct Mesh {
    /*  Mesh Data  */
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub VAO: u32,

    /*  Render data  */
    VBO: u32,
    EBO: u32,
}

impl Mesh {
    pub unsafe fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
        let mut mesh = Mesh {
            vertices, indices, textures,
            VAO: 0, VBO: 0, EBO: 0
        };

        // now that we have all the required data, set the vertex buffers and its attribute pointers.
        mesh.setupMesh();
        mesh
    }

    unsafe fn setupMesh(&mut self) {
        // create buffers/arrays
        gl::GenVertexArrays(1, &mut self.VAO);
        gl::GenBuffers(1, &mut self.VBO);
        gl::GenBuffers(1, &mut self.EBO);

        gl::BindVertexArray(self.VAO);
        // load data into vertex buffers
        gl::BindBuffer(gl::ARRAY_BUFFER, self.VBO);
        // A great thing about structs with repr(C) is that their memory layout is sequential for all its items.
        // The effect is that we can simply pass a pointer to the struct and it translates perfectly to a glm::vec3/2 array which
        // again translates to 3/2 floats which translates to a byte array.
        let size = (self.vertices.len() * size_of::<GLfloat>()) as isize;
        let data = &self.vertices[0] as *const Vertex as *const c_void;
        gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.EBO);
        let size = (self.indices.len() * size_of::<u32>()) as isize;
        let data = &self.indices[0] as *const u32 as *const c_void;
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

        // set the vertex attribute pointers
        let size = size_of::<Vertex>() as i32;
        // vertex Positions
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size, ptr::null());
        // vertex normals
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Normal) as *const c_void);
        // vertex texture coords
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, TexCoords) as *const c_void);
        // vertex tangent
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Tangent) as *const c_void);
        // vertex bitangent
        gl::EnableVertexAttribArray(4);
        gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, Bitangent) as *const c_void);

        gl::BindVertexArray(0);
    }
}
