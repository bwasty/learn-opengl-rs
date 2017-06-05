# learn-opengl-rs
Rust port of https://github.com/JoeyDeVries/LearnOpenGL

You should be able to follow the tutorials on https://learnopengl.com/ with this - the code structure has been kept similar to the original C++ where possible.
> This also means it's not necessarily the most idiomatic Rust code. For example, all OpenGL calls are "raw" and wrapped in `unsafe` blocks.

Run individual tutorials like this:
`cargo run --bin gs_1_2_hello_window_clear`

### Ported so far
| Original | Port |
| --- | --- |
|`/src/1.getting_started/1.1.hello_window/` | See 1.2 (same with just 2 code lines more) |
| `/src/1.getting_started/1.2.hello_window_clear/` | `/src/bin/gs_1_2_hello_window_clear.rs` |
| `/src/1.getting_started/2.1.hello_triangle/` | `/src/bin/gs_2_1_hello_triangle.rs` |
| `/src/1.getting_started/2.2.hello_triangle_indexed/` | `/src/bin/gs_2_2_hello_triangle_indexed.rs` |
| `/src/1.getting_started/2.3.hello_triangle_exercise1/` | `/src/bin/gs_2_3_hello_triangle_exercise1.rs` |
| `/src/1.getting_started/2.4.hello_triangle_exercise2/` | `/src/bin/gs_2_4_hello_triangle_exercise2.rs` |
| `/src/1.getting_started/2.5.hello_triangle_exercise3/` | `/src/bin/gs_2_5_hello_triangle_exercise3.rs` |
| `/src/1.getting_started/3.1.shaders_uniform/` | `/src/bin/gs_3_1_shaders_uniform.rs` |
| `/src/1.getting_started/3.2.shaders_interpolation/` | `/src/bin/gs_3_2_shaders_interpolation.rs` |
| `/src/1.getting_started/3.3.shaders_class/` | `/src/bin/gs_3_3_shaders_class.rs` <br> `/src/bin/shaders.rs`
