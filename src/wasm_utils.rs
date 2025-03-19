use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;
use web_sys::console::log_1;
use web_sys::*;

#[wasm_bindgen(inline_js = "export function print_string(a){ term.write(a); }")]
extern "C"{
    pub fn print_string(a:String);
}

#[wasm_bindgen(inline_js = "export function get_char(){ if(recv_fifo.length != 0){ return recv_fifo.shift(); }else{ return 0; } }")]
extern "C"{
    pub fn get_char() -> u32;
}

#[wasm_bindgen(inline_js = "export function clear_image_data(){ image_avail = false; image_data = 0; }")]
extern "C"{
    pub fn clear_image_data();
}

#[wasm_bindgen(inline_js = "export function get_image_data(){ return image_data; }")]
extern "C"{
    pub fn get_image_data() -> js_sys::Uint8Array;
}

#[wasm_bindgen(inline_js = "export function check_image_avail(){ return image_avail; }")]
extern "C"{
    pub fn check_image_avail() -> js_sys::Boolean;
}

#[wasm_bindgen(inline_js = "export function get_requested_flash_capacity(){ return document.getElementById('flash_capacity').value; }")]
extern "C"{
    pub fn get_requested_flash_capacity() -> u32;
}


pub fn log(s: &String) {
    log_1(&JsValue::from(s));
}

#[wasm_bindgen]
pub async fn sleep(ms: i32){
    let p : js_sys::Promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    });
    match wasm_bindgen_futures::JsFuture::from(p).await {
        Ok( _d) => {}
        _ => { return ; }
    }
}
