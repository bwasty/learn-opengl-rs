extern crate gl;
extern crate image;
extern crate cgmath;
extern crate tobj;

mod shader;
mod macros;
mod camera;
mod mesh;
mod model;
mod utils;

#[cfg(feature = "chapter-1")]
mod _1_getting_started;
#[cfg(feature = "chapter-1")]
use _1_getting_started::*;

#[cfg(feature = "chapter-2")]
mod _2_lighting;
#[cfg(feature = "chapter-2")]
use _2_lighting::*;

#[cfg(feature = "chapter-3")]
mod _3_model_loading;
#[cfg(feature = "chapter-3")]
use _3_model_loading::*;

#[cfg(feature = "chapter-4")]
mod _4_advanced_opengl;
#[cfg(feature = "chapter-4")]
use _4_advanced_opengl::*;

#[cfg(feature = "chapter-7")]
mod _7_in_practice;
#[cfg(feature = "chapter-7")]
use _7_in_practice::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with the number of the tutorial, e.g. `1_1_2` for _1_2_hello_window_clear.rs");
        std::process::exit(1);
    }
    let tutorial_id = &args[1];

    #[cfg(feature = "chapter-1")]
    match tutorial_id.as_str() {
        "1_1_1" => main_1_1_1(),
        "1_1_2" => main_1_1_2(),
        "1_2_1" => main_1_2_1(),
        "1_2_2" => main_1_2_2(),
        "1_2_3" => main_1_2_3(),
        "1_2_4" => main_1_2_4(),
        "1_2_5" => main_1_2_5(),
        "1_3_1" => main_1_3_1(),
        "1_3_2" => main_1_3_2(),
        "1_3_3" => main_1_3_3(),
        "1_4_1" => main_1_4_1(),
        "1_4_2" => main_1_4_2(),
        "1_5_1" => main_1_5_1(),
        "1_6_1" => main_1_6_1(),
        "1_6_2" => main_1_6_2(),
        "1_6_3" => main_1_6_3(),
        "1_7_1" => main_1_7_1(),
        "1_7_2" => main_1_7_2(),
        "1_7_3" => main_1_7_3(),
        "1_7_4" => main_1_7_4(),
        _       => ()
    }

    #[cfg(feature = "chapter-2")]
    match tutorial_id.as_str() {
        "2_1" => main_2_1(),
        "2_2_1" => main_2_2_1(),
        "2_2_2" => main_2_2_2(),
        "2_3_1" => main_2_3_1(),
        "2_4_1" => main_2_4_1(),
        "2_4_2" => main_2_4_2(),
        "2_5_1" => main_2_5_1(),
        "2_5_2" => main_2_5_2(),
        "2_5_3" => main_2_5_3(),
        "2_5_4" => main_2_5_4(),
        "2_6"   => main_2_6(),
        _       => ()
    }

    #[cfg(feature = "chapter-3")]
    match tutorial_id.as_str() {
        "3_1" => main_3_1(),
        _     => ()
    }

    #[cfg(feature = "chapter-4")]
    match tutorial_id.as_str() {
        "4_1_1" => main_4_1_1(),
        _       => ()
    }

    #[cfg(feature = "chapter-7")]
    match tutorial_id.as_str() {
        "7_1" => main_7_1(),
        _     => ()
    }

    // TODO!: can't detect properly when to print this with the features...
    println!("Unknown tutorial id");
}
