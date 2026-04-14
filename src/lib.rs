mod lexer;
mod parser;
mod interpreter;

mod utils;

use wasm_bindgen::prelude::*;

use crate::{interpreter::Interpreter, lexer::Lexer};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn wasm_init() {
    web_sys::console::log_1(&format!("WASM running").into());

    let mut lexer = Lexer::new("int main() { print(); }");
    lexer.read();

    let _ = Interpreter::new();
}
