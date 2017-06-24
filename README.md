# learn-opengl-rs
Rust port of https://github.com/JoeyDeVries/LearnOpenGL

You should be able to follow the tutorials at https://learnopengl.com/ with this - the code structure has been kept similar to the original C++ wherever possible.
> This also means it's not necessarily the most idiomatic Rust code. For example, some standard naming conventions lints are disabled and all OpenGL calls are "raw" and wrapped in `unsafe` blocks.

Run individual tutorials like this:
`cargo run 1_3_2` (for `/src/_1_getting_started/_3_2_shaders_interpolation.rs`)

## Chapters
### [1. Getting started](src/_1_getting_started)
* status: complete*
* Notes:
    - You can mostly ignore the setup instructions at [Getting-started/Creating-a-window](https://learnopengl.com/#!Getting-started/Creating-a-window). Just create a new project with `cargo` and copy the dependencies section from [Cargo.toml](Cargo.toml). Only `glfw-rs` might need some more setup, see [here](https://github.com/PistonDevelopers/glfw-rs#using-glfw-rs) for details. You can also use [glutin](https://github.com/tomaka/glutin) (a pure Rust alternative to GLFW), but the API is a bit different, so following the tutorials might not be as straight-forward.
    - You might be tempted to use [glium](https://github.com/glium/glium) instead of raw OpenGL. I'd recommend against that, at least in the beginning, to get a good understanding of how OpenGL really works. Also, glium is not actively maintained at the moment.

### [2. Lighting](src/_2_lighting)
* status: complete*

### [3. Model loading](src/_3_model_loading)
* status: complete
* Notes
    - For simplicity `tobj` is used instead of `assimp` (simpler interface, pure Rust and later tutorials only load OBJ files anyway). For alternatives see [here](http://arewegameyet.com/categories/3dformatloader.html) and [here](https://crates.io/search?q=assimp).
    - The `image` crate is quite slow in debug mode - loading the nanosuit textures takes so much time that it can be faster to use release mode (including compile time).
### [7. In Practice](src/_7_in_practice)
* status: `Debugging` complete (the other two are not in the repo)

### TODO
### 4. Advanced OpenGL
### 5. Advanced Lighting
### 6. PBR

----
\* exercises mostly omitted. You can look up the solutions in the original C++ source.

----
#### A note about the code organization
Originally each tutorial was a separate executable (using `src/bin` and `cargo run --bin <name>`. This didn't play very well with the `RLS` and `clippy` (-> rust-lang-nursery/rls#132). Now all are integrated into the main binary, which leads to long compile times. A workaround for this is to comment out all tutorials except the one you're working on in `main.rs` and the respective `mod.rs` of the chapter.
