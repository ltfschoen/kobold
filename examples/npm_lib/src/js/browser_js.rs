// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use wasm_bindgen::prelude::*;
use gloo_console::log;

use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Uint8Array;
use js_sys::Array;

use crate::js::interfaces::browser_js_spec_npm_lib as connect;

#[wasm_bindgen]
pub async fn run_npm_lib() -> Result<JsValue, JsValue> {
    let hash = connect::kobold_npm_lib().await; // Uint8Array

    let hash_u8a = &hash.as_ref().unwrap();
    let buffer: &JsValue = hash_u8a;
    let array_buf: Uint8Array = Uint8Array::new(buffer);
    // can't pass Vec<u8> into `into_serde` since it returns error:
    // the trait `JsCast` is not implemented for `Vec<u8>`
    // so we convert it to an Array since it supports that
    let bytes: Vec<u8> = array_buf.to_vec();

    // it's possible to convert back again with:
    // let array_again: Uint8Array = Uint8Array::from(&bytes);

    // structs are a Rust data type, and so you have to manually use
    // `JsValue::from` to convert them into a JS data type
    // (the same is true for other Rust data types like &str, i32, etc.)
    // source: https://github.com/rustwasm/wasm-bindgen/issues/111#issuecomment-542968427
    let array: Array = bytes.clone().into_iter().copy_from().map(JsValue::from).collect();
    log!("array {:?}", &array);

    // let bytes_serded: Uint8Array = JsValue::from(array).into_serde().unwrap();
    
    // let test = JsValue::from(array).into_serde::<Uint8Array>().unwrap();
    // log!("test {:?}", &test);

    // JsValue::into_serde(&bytes).unwrap();
    // log!("bytes_serded {:?}", &bytes_serded);
    // let hash_hex = std::str::from_utf8(bytes.as_ref());
    // let hash_hex = unsafe {
    //     std::str::from_utf8_unchecked(bytes.clone().as_ref())
    // };
    // log!("hash_hex {:?}", hash_hex.to_string());
    // log!("hash_hex {:?}", &hash_hex.unwrap().to_string());
    

    // log!(&format_args!("connect::kobold_npm_lib() {:?}", hash.as_ref()?).to_string());
    hash
}
