# learn-opengl-rs
Rust port of https://github.com/JoeyDeVries/LearnOpenGL

You should be able to follow the tutorials on https://learnopengl.com/ with this - the code structure has been kept similar to the original C++ wherever possible.
> This also means it's not necessarily the most idiomatic Rust code. For example, some standard naming conventions lints are disabled and all OpenGL calls are "raw" and wrapped in `unsafe` blocks.

Run individual tutorials like this:
`cargo run 1_1_2` (for `/src/_1_getting_started/_1_2_hello_window_clear.rs`)

### Ported so far
| Original | Port |
| --- | --- |
|`1.getting_started/1.1.hello_window/` | See 1.2 (same with just 2 code lines more) |
| `1.getting_started/1.2.hello_window_clear/` | `_1_getting_started/_1_2_hello_window_clear.rs` |
| `1.getting_started/2.1.hello_triangle/` | `_1_getting_started/_2_1_hello_triangle.rs` |
| `1.getting_started/2.2.hello_triangle_indexed/` | `_1_getting_started/_2_2_hello_triangle_indexed.rs` |
| `1.getting_started/2.3.hello_triangle_exercise1/` | `_1_getting_started/_2_3_hello_triangle_exercise1.rs` |
| `1.getting_started/2.4.hello_triangle_exercise2/` | `_1_getting_started/_2_4_hello_triangle_exercise2.rs` |
| `1.getting_started/2.5.hello_triangle_exercise3/` | `_1_getting_started/_2_5_hello_triangle_exercise3.rs` |
| `1.getting_started/3.1.shaders_uniform/` | `_1_getting_started/_3_1_shaders_uniform.rs` |
| `1.getting_started/3.2.shaders_interpolation/` | `_1_getting_started/_3_2_shaders_interpolation.rs` |
| `1.getting_started/3.3.shaders_class/` | `_1_getting_started/_3_3_shaders_class.rs` <br> `shaders.rs`
| `1.getting_started/4.1.textures/` | `_1_getting_started/_4_1_textures.rs`
| `1.getting_started/4.2.textures_combined/` | `_1_getting_started/_4_2_textures_combined.rs`
| `1.getting_started/5.1.transforms/` | `_1_getting_started/_5_1_transform.rs`
| `1.getting_started/6.1.coordinate_systems/` | `_1_getting_started/_6_1_coordinate_systems.rs`
| `1.getting_started/6.2.coordinate_systems_depth/` | `_1_getting_started/_6_2_coordinate_systems_depth.rs`
| `1.getting_started/6.3.coordinate_systems_multiple/` | `_1_getting_started/_6_3_coordinate_systems_multiple.rs`
| `1.getting_started/7.1.camera_circle/` | `_1_getting_started/_7_1_camera_circle.rs`
| `1.getting_started/7.2.camera_keyboard_dt/` | `_1_getting_started/_7_2_camera_keyboard_dt.rs`
| `1.getting_started/7.3.camera_mouse_zoom/` | `_1_getting_started/_7_3_camera_mouse_zoom.rs`
| `1.getting_started/7.3.camera_class/` | `_1_getting_started/_7_3_camera_class.rs` <br> `camera.rs`
