// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use helper::SerializedNumber;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use wast::{
    self,
    core::{Func, ModuleField},
    parser::{self, ParseBuffer},
    token::Id,
    Wat,
};

mod error;
mod helper;
mod instruction;
mod marker;
mod validator;

use error::{WatError, WatResult};
use instruction::{InputOutput, SerializedInstruction, SerializedInstructionTree};
use validator::Validator;

use marker::SerializableWatType;

/// A basic Wa(s)t Function
///
/// ## Note:
/// Does not work with imported functions, as it assumes nothing about other modules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WastFunc {
    info: instruction::InputOutput,
    locals: Vec<(Option<String>, SerializableWatType)>,
    block: SerializedInstructionTree,
}

impl WastFunc {
    pub fn set_name_from_number(&mut self, index: usize) {
        self.info.index = Some(index.to_string());
    }

    pub fn name(&self) -> Option<String> {
        self.info.index.clone()
    }
}

impl TryFrom<&Func<'_>> for WastFunc {
    type Error = error::WatError;

    fn try_from(value: &Func<'_>) -> Result<Self, Self::Error> {
        // dbg!(value.id);
        // dbg!(value.ty.index);
        let mut info = InputOutput::try_from(&value.ty)?;
        if let Some(id) = value.id {
            info.set_name_if_none(id.name());
        }

        match &value.kind {
            wast::core::FuncKind::Import(_) => Err(error::WatError::unimplemented_error(
                "Import functions are not supported yet.",
            )),
            wast::core::FuncKind::Inline { locals, expression } => Ok(WastFunc {
                info,
                locals: locals
                    .iter()
                    .map(|l| match SerializableWatType::try_from(l.ty) {
                        Ok(ty) => Ok((l.id.map(|i| i.name().to_string()), ty)),
                        Err(err) => Err(err),
                    })
                    .collect::<Result<Vec<_>, error::WatError>>()?,
                block: (*expression.instrs).try_into()?,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type, derive_more::Display)]
pub enum NumLocationKind {
    Function,
    Global,
    Memory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GlobalData {
    name: String,
    typ: SerializableWatType,
    is_mutable: bool,
    val: SerializedNumber,
}

impl GlobalData {
    pub fn try_new(
        name: String,
        gtyp: SerializableWatType,
        is_mutable: bool,
        instructions: Vec<SerializedInstruction>,
    ) -> WatResult<Self> {
        match instructions.as_slice() {
            [SerializedInstruction::Const { typ, value }] if typ == &gtyp => Ok(Self {
                name,
                typ: gtyp,
                is_mutable,
                val: *value,
            }),
            [] => Err(WatError::no_instruction_provided("Const")),
            _ => Err(WatError::non_initializer_expression()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MemoryData {
    name: String,
    min: SerializedNumber,
    max: SerializedNumber,
    is_32: bool,
    is_shared: bool,
    data: Vec<u8>,
}

impl MemoryData {
    pub fn new(
        name: String,
        min: u64,
        max: Option<u64>,
        is_32: bool,
        is_shared: bool,
        data: Vec<u8>,
    ) -> Self {
        let min = min.into();
        let max = max.into();
        Self {
            name,
            min,
            max,
            is_32,
            is_shared,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct InterpreterStructure {
    pub(crate) name: String,
    pub(crate) exported: HashMap<String, (NumLocationKind, u32)>,
    pub(crate) globals: Vec<GlobalData>,
    pub(crate) memory: Vec<MemoryData>,
    pub(crate) func: Vec<WastFunc>,
}

impl InterpreterStructure {
    /// Try to create a new interpreter structure
    pub fn try_new(_text: &str, fields: &[ModuleField], name: &Option<Id>) -> WatResult<Self> {
        let mut exported = HashMap::new();
        let mut globals = Vec::new();
        let mut memory = Vec::new();
        let mut func = Vec::new();
        // let mut start = 0;
        for (_i, field) in fields.iter().enumerate() {
            match field {
                ModuleField::Type(_) => todo!("Type field not implemented"),
                ModuleField::Rec(_) => todo!("Rec field not implemented"),
                ModuleField::Import(_) => todo!("Import field not implemented"),
                ModuleField::Func(f) => {
                    exported.extend(f.exports.names.iter().map(|name| {
                        (
                            name.to_string(),
                            (NumLocationKind::Function, func.len() as u32),
                        )
                    }));
                    let mut function = WastFunc::try_from(f)?;
                    if function.name().is_none() {
                        function.set_name_from_number(func.len())
                    };
                    func.push(function);
                }
                ModuleField::Table(_) => todo!("Table field not implemented"),
                ModuleField::Memory(m) => {
                    exported.extend(m.exports.names.iter().map(|name| {
                        (
                            name.to_string(),
                            (NumLocationKind::Memory, func.len() as u32),
                        )
                    }));
                    match &m.kind {
                        wast::core::MemoryKind::Import { import: _, ty: _ } => {
                            Err(error::WatError::unimplemented_error(
                                "Imported memory not yet implemented.",
                            ))?
                        }
                        wast::core::MemoryKind::Normal(mt) => match mt {
                            wast::core::MemoryType::B32 { limits, shared } => {
                                memory.push(MemoryData::new(
                                    m.id.map(|id| id.name().to_string()).unwrap_or_default(),
                                    limits.min as u64,
                                    limits.max.map(|n| n as u64),
                                    true,
                                    *shared,
                                    Vec::new(),
                                ));
                            }
                            wast::core::MemoryType::B64 { limits, shared } => {
                                memory.push(MemoryData::new(
                                    m.id.map(|id| id.name().to_string()).unwrap_or_default(),
                                    limits.min,
                                    limits.max,
                                    false,
                                    *shared,
                                    Vec::new(),
                                ));
                            }
                        },
                        wast::core::MemoryKind::Inline { is_32: _, data: _ } => {
                            Err(error::WatError::unimplemented_error(
                                "Inline memory not yet implemented.",
                            ))?
                        }
                    }
                }
                ModuleField::Global(g) => {
                    exported.extend(g.exports.names.iter().map(|name| {
                        (
                            name.to_string(),
                            (NumLocationKind::Global, func.len() as u32),
                        )
                    }));
                    match &g.kind {
                        wast::core::GlobalKind::Import(_) => {
                            Err(error::WatError::unimplemented_error(
                                "Imported globals not yet implemented.",
                            ))?
                        }
                        wast::core::GlobalKind::Inline(e) => {
                            globals.push(GlobalData::try_new(
                                g.id.map(|id| id.name().to_string()).unwrap_or_default(),
                                g.ty.ty.try_into()?,
                                g.ty.mutable,
                                e.instrs
                                    .iter()
                                    .map(|ins| ins.try_into())
                                    .collect::<Result<_, _>>()?,
                            )?);
                        }
                    }
                }
                ModuleField::Export(e) => match e.kind {
                    wast::core::ExportKind::Func => {
                        for (i, f) in func.iter().enumerate() {
                            if f.name()
                                .as_ref()
                                .is_some_and(|item| item == &instruction::index_to_string(&e.item))
                            {
                                exported.insert(
                                    e.name.to_string(),
                                    (NumLocationKind::Function, i as u32),
                                );
                                break;
                            }
                        }
                    }
                    wast::core::ExportKind::Table => todo!("Export Tables not implemented"),
                    wast::core::ExportKind::Memory => {
                        for (i, m) in memory.iter().enumerate() {
                            if m.name == instruction::index_to_string(&e.item) {
                                exported.insert(
                                    e.name.to_string(),
                                    (NumLocationKind::Memory, i as u32),
                                );
                                break;
                            }
                        }
                    }
                    wast::core::ExportKind::Global => {
                        for (i, g) in globals.iter().enumerate() {
                            if g.name == instruction::index_to_string(&e.item) {
                                exported.insert(
                                    e.name.to_string(),
                                    (NumLocationKind::Global, i as u32),
                                );
                                break;
                            }
                        }
                    }
                    wast::core::ExportKind::Tag => todo!("Export Tags not implemented"),
                },
                ModuleField::Start(_) => todo!("Start field not implemented"),
                ModuleField::Elem(_) => todo!("Element field not implemented"),
                ModuleField::Data(_) => todo!("Data field not implemented"),
                ModuleField::Tag(_) => todo!("Tag field not implemented"),
                ModuleField::Custom(_) => todo!("Custom field not implemented"),
            }
        }
        let interp_struct = InterpreterStructure {
            name: name.map(|id| id.name().to_string()).unwrap_or_default(),
            exported,
            globals,
            memory,
            func,
        };
        // interp_struct.validate()?;
        Ok(interp_struct)
    }

    /// Validate that the structure is correct, check all types match, and stack flow is correct.
    pub fn validate(&self) -> WatResult<()> {
        // TODO: Remove the need for .clone()
        let funcs: HashMap<_, _> = self
            .func
            .iter()
            .enumerate()
            .flat_map(|(i, f)| {
                let params: Vec<_> = f.info.input.iter().map(|(_, t)| *t).collect();
                let results = &f.info.output;
                if let Some(name) = f.name() {
                    [
                        (i.to_string(), (params.clone(), results.clone())),
                        (name, (params, results.clone())),
                    ]
                } else {
                    [
                        (i.to_string(), (params.clone(), results.clone())),
                        (i.to_string(), (params, results.clone())),
                    ]
                }
            })
            .collect();
        for func in &self.func {
            let mut validator = Validator::new(
                self.globals
                    .iter()
                    .enumerate()
                    .flat_map(|(i, g)| {
                        [
                            (i.to_string(), (g.is_mutable, g.typ)),
                            (g.name.clone(), (g.is_mutable, g.typ)),
                        ]
                    })
                    .collect(),
                func.info
                    .input
                    .iter()
                    .chain(func.locals.iter())
                    .enumerate()
                    .flat_map(|(i, l)| {
                        if let Some(name) = l.0.clone() {
                            [(i.to_string(), l.1), (name, l.1)]
                        } else {
                            [(i.to_string(), l.1), (i.to_string(), l.1)]
                        }
                    })
                    .collect(),
                funcs.clone(),
                self.memory.iter().map(|m| m.name.clone()).collect(),
                func.info.output.clone(),
            );
            dbg!(&func.block);
            // for instruction in func.block.get_root() {
            //     validator.process(instruction)?;
            // }
        }
        Ok(())
    }
}

#[tauri::command]
#[specta::specta]
fn transform(text: &str) -> error::WatResult<InterpreterStructure> {
    // Note: New only builds the buffer and is currently infallible
    let buffer = ParseBuffer::new(text).map_err(WatError::parsing_error)?;
    // Combined lexing and parsing step
    let mut module = match parser::parse::<Wat>(&buffer).map_err(WatError::parsing_error)? {
        Wat::Module(m) => m,
        Wat::Component(_) => {
            return Err(error::WatError::unimplemented_error(
                "Cannot compile components currently.",
            ));
        }
    };
    // dbg!(&module);
    let final_result = match module.kind {
        wast::core::ModuleKind::Text(ref fields) => {
            InterpreterStructure::try_new(text, fields, &module.id)
        }
        wast::core::ModuleKind::Binary(_) => Err(error::WatError::unimplemented_error(
            "Unimplemented Error: Cannot binary type currently.",
        )),
    };
    // Resolve and immediately throw-away
    let _ = module.resolve().map_err(WatError::resolution_error)?;
    // Print for debug purposes, it does change module, so need to resolve name separately.
    // println!("{}", wasmprinter::print_bytes(module.encode().unwrap()).unwrap());
    final_result
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![transform])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod export_bindings {
    //! This module is only for exporting binding for TypeScript
    use super::*;
    #[test]
    fn export_bindings() {
        dbg!(tauri_specta::ts::export(
            specta::collect_types![transform],
            "../src/lib/bindings.ts"
        ))
        .unwrap();
    }
}
