// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::array;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;
use wast::core::{Expression, Func, FunctionType, Local, Module, ModuleField, ValType};
use wast::kw::param;
use wast::parser::{self, ParseBuffer, Parser, Result as WastResult};
use wast::token::{Id, Span};
use wast::{Error as WastError, Wat};

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

#[derive(Debug, Clone, Serialize, Type)]
pub struct ErrorHolder {
    offset: usize,
    message: String,
}

impl From<WastError> for ErrorHolder {
    fn from(value: WastError) -> Self {
        value.span().offset();

        ErrorHolder {
            offset: value.span().offset(),
            message: value.message(),
        }
    }
}

pub fn unimplemented_error(msg: &str) -> WastError {
    WastError::new(Span::from_offset(0), format!("Unimplemented Error: {msg}"))
}

/// All Wat types that can be serialized.
///
/// ## Limitations
/// All except [ValType::Ref] are supported, but must explicity convert.
#[derive(Debug, Clone, Copy, Serialize, Type)]
pub enum SerializableWatType {
    I32,
    I64,
    F32,
    F64,
    V128,
}

impl<'a> TryFrom<ValType<'a>> for SerializableWatType {
    type Error = WastError;

    fn try_from(value: ValType) -> Result<Self, Self::Error> {
        match value {
            ValType::I32 => Ok(SerializableWatType::I32),
            ValType::I64 => Ok(SerializableWatType::I64),
            ValType::F32 => Ok(SerializableWatType::F32),
            ValType::F64 => Ok(SerializableWatType::F64),
            ValType::V128 => Ok(SerializableWatType::V128),
            ValType::Ref(_) => Err(unimplemented_error("Cannot use Ref type")),
        }
    }
}

/// A basic Wa(s)t Function
///
/// ## Note:
/// Does not work with imported functions, as it assumes nothing about other modules
#[derive(Debug, Clone, Serialize, Type)]
pub struct WastFunc {
    name: Option<String>,
    parameters: Vec<(Option<String>, SerializableWatType)>,
    locals: Vec<(Option<String>, SerializableWatType)>,
    body: Vec<String>,
    result: Vec<SerializableWatType>,
}

impl TryFrom<&Func<'_>> for WastFunc {
    type Error = WastError;

    fn try_from(value: &Func<'_>) -> Result<Self, Self::Error> {
        let name = value.id.map(|i| i.name().to_string());
        if value.ty.index.is_some() {
            return Err(unimplemented_error(
                "Index value should not be assigned for functions, I believe",
            ));
        }
        let (parameters, result) = match &value.ty.inline {
            Some(FunctionType { params, results }) => (
                params
                    .into_iter()
                    .map(|p| match SerializableWatType::try_from(p.2) {
                        Ok(ty) => Ok((p.0.map(|i| i.name().to_string()), ty)),
                        Err(err) => Err(err),
                    })
                    .collect::<Result<Vec<_>, WastError>>()?,
                results
                    .into_iter()
                    // TODO: Remove clone for r
                    .map(|r| SerializableWatType::try_from(r.clone()))
                    .collect::<Result<Vec<_>, WastError>>()?,
            ),
            None => (Vec::default(), Vec::default()),
        };
        match &value.kind {
            wast::core::FuncKind::Import(_) => {
                Err(unimplemented_error("Import functions are not supported"))
            }
            wast::core::FuncKind::Inline { locals, expression } => Ok(WastFunc {
                name,
                parameters,
                locals: locals
                    .into_iter()
                    .map(|l| match SerializableWatType::try_from(l.ty) {
                        Ok(ty) => Ok((l.id.map(|i| i.name().to_string()), ty)),
                        Err(err) => Err(err),
                    })
                    .collect::<Result<Vec<_>, WastError>>()?,
                body: expression
                    .instrs
                    .into_iter()
                    .map(|i| format!("{i:?}"))
                    .collect(),
                result,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub enum NumLocationKind {
    Function,
    Global,
    Memory,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct InterpreterStructure {
    name: String,
    exported: HashMap<String, (NumLocationKind, u32)>,
    globals: Vec<String>,
    memory: Vec<String>,
    func: Vec<WastFunc>,
}

impl InterpreterStructure {
    pub fn try_new(text: &str, fields: &Vec<ModuleField>, name: &Option<Id>) -> WastResult<Self> {
        let mut exported = HashMap::new();
        let mut func = Vec::new();
        let mut start = 0;
        for (i, field) in fields.iter().enumerate() {
            match field {
                ModuleField::Type(_) => todo!("Type field not implemented"),
                ModuleField::Rec(_) => todo!("Rec field not implemented"),
                ModuleField::Import(_) => todo!("Import field not implemented"),
                ModuleField::Func(f) => {
                    dbg!(i, f);

                    for name in &f.exports.names {
                        exported.insert(
                            name.to_string(),
                            (NumLocationKind::Function, func.len() as u32),
                        );
                    }
                    func.push(WastFunc::try_from(f).map_err(|e| e.into())?)
                }
                ModuleField::Table(_) => todo!("Table field not implemented"),
                ModuleField::Memory(m) => {
                    dbg!(i, m);
                }
                ModuleField::Global(g) => {
                    dbg!(i, g);
                    // match g.kind {}
                }
                ModuleField::Export(e) => {
                    dbg!(i, e);
                }
                ModuleField::Start(_) => todo!("Start field not implemented"),
                ModuleField::Elem(_) => todo!("Element field not implemented"),
                ModuleField::Data(d) => {
                    dbg!(i, d);
                }
                ModuleField::Tag(_) => todo!("Tag field not implemented"),
                ModuleField::Custom(_) => todo!("Custom field not implemented"),
            }
        }
        Ok(InterpreterStructure {
            name: name.map(|id| id.name().to_string()).unwrap_or_default(),
            exported,
            globals: Vec::default(),
            memory: Vec::default(),
            func,
        })
    }
}

fn create_buffer<'a>(text: &'a str) -> WastResult<ParseBuffer<'a>> {
    ParseBuffer::new(text)
}

fn convert_buffer<'a>(buffer: &'a ParseBuffer<'a>) -> WastResult<Wat<'a>> {
    parser::parse::<'a, Wat<'a>>(buffer)
}

#[tauri::command]
#[specta::specta]
fn transform(text: &str) -> Result<InterpreterStructure, ErrorHolder> {
    let buf = create_buffer(text)?;
    let module = convert_buffer(&buf)?;
    match module {
        Wat::Module(m) => match &m.kind {
            wast::core::ModuleKind::Text(fields) => {
                Ok(InterpreterStructure::try_new(text, fields, &m.id)?)
            }
            wast::core::ModuleKind::Binary(_) => Err(unimplemented_error(
                "Unimplemented Error: Cannot binary type currently.",
            )
            .into()),
        },
        Wat::Component(_) => {
            Err(unimplemented_error("Cannot compile components currently.").into())
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![transform])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_bindings() {
        tauri_specta::ts::export(specta::collect_types![transform], "../src/bindings.ts").unwrap();
    }
}
