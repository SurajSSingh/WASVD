// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

const WAT_EXAMPLE: &'static str = r#"(module
    (func $getAnswer (result i32) i32.const 42)
    (func (export "getAnswerPlus1") (result i32)
      call $getAnswer
      i32.const 1
      i32.add
    )
    (func (export "muladdsqr") (param $a i32) (param $b i32) (param $c i32) (result i32)
        (local $i i32)
        (i32.mul (local.get $a) (local.get $b))
        (i32.add (local.get $c))
        local.tee $i
        local.get $i
        i32.mul
    )
  )"#;

use wast::parser::{self, ParseBuffer, Result as WastResult};
use wast::Wat;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn create_buffer<'a>(text: &'a str) -> WastResult<ParseBuffer<'a>> {
    ParseBuffer::new(text)
}

fn convert_buffer<'a>(buffer: &'a ParseBuffer<'a>) -> WastResult<Wat<'a>> {
    parser::parse::<'a, Wat<'a>>(buffer)
}

#[tauri::command]
fn test() -> String {
    let buf = match create_buffer(WAT_EXAMPLE) {
        Ok(buf) => buf,
        Err(err) => return err.to_string(),
    };
    let module = match convert_buffer(&buf) {
        Ok(wat) => wat,
        Err(err) => return err.to_string(),
    };
    format!("{:#?}", module)
}

#[tauri::command]
fn transform(text: &str) -> String {
    let buf = match create_buffer(text) {
        Ok(buf) => buf,
        Err(err) => return err.to_string(),
    };
    let module = match convert_buffer(&buf) {
        Ok(wat) => wat,
        Err(err) => return err.to_string(),
    };
    format!("{:?}", module)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
