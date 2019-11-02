extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// this function changes a value by reference (borrowing and change)
#[wasm_bindgen]
pub fn alter(a: &mut [u8]) {
    a[1] = 12;
}

