extern crate gl;
extern crate image;
extern crate cgmath;

mod shader;
mod utils;
mod camera;

mod _1_getting_started;
use _1_getting_started::*;
mod _2_lighting;
use _2_lighting::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with the number of the tutorial, e.g. `1_1_2` for _1_2_hello_window_clear.rs");
        std::process::exit(1);
    }
    let tutorial_id = &args[1];
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
        "2_6" => main_2_6(),
        _ => println!("Unkown tutorial id"),
    }

}
