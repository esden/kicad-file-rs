use chumsky::prelude::*;
use std::{env, fs};

use kicad_sexp::{parser, pretty_print};

fn main() {
    let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument")).expect("Failed to read file");

    let result = parser().parse(src.trim()).into_result();

    //println!("{:?}", result);
    match result {
        Ok(sexps) => {
            println!("Parse success. We got:");
            pretty_print(&sexps);
            println!();
        },
        Err(err) => {
            println!("Parse failed. Errors are:");
            for e in err {
                println!("{}", e);
            }
        },
    }
}