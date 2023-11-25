//! This module focuses on errors for WAT conversion.
use std::{fmt::Display, ops::Range};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{marker::SerializableWatType, NumLocationKind};

pub type WatResult<T> = Result<T, WatError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, derive_more::Display)]
pub enum ErrorStage {
    Parsing,
    TypeChecking,
    NameResolving,
    Unimplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, derive_more::Error)]
pub struct WatError {
    span: Option<Range<usize>>,
    stage: ErrorStage,
    message: Option<String>,
}

impl Display for WatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.span, &self.message) {
            (None, None) => f.write_fmt(format_args!("[{} Error]", self.stage)),
            (None, Some(msg)) => f.write_fmt(format_args!("[{} Error]: {}", self.stage, msg)),
            (Some(Range { start, end }), None) => {
                f.write_fmt(format_args!("[{} Error@{}-{}]", self.stage, start, end))
            }
            (Some(Range { start, end }), Some(msg)) => f.write_fmt(format_args!(
                "[{} Error@{}-{}]: {}",
                self.stage, start, end, msg
            )),
        }?;
        Ok(())
    }
}

impl WatError {
    pub fn unimplemented_error(msg: &str) -> Self {
        Self {
            span: None,
            stage: ErrorStage::Unimplemented,
            message: Some(msg.to_string()),
        }
    }

    pub fn invalid_instruction(expected_type: &str, instruction: &wast::core::Instruction) -> Self {
        Self {
            span: None,
            stage: ErrorStage::Parsing,
            message: Some(format!(
                "Not a valid {expected_type} instruction: {instruction:?}"
            )),
        }
    }

    pub fn parsing_error(value: wast::Error) -> Self {
        let offset = value.span().offset();
        Self {
            span: Some(offset..offset + 1),
            stage: ErrorStage::Parsing,
            message: Some(value.message()),
        }
    }

    pub fn resolution_error(value: wast::Error) -> Self {
        let offset = value.span().offset();
        Self {
            span: Some(offset..offset + 1),
            stage: ErrorStage::NameResolving,
            message: Some(value.message()),
        }
    }

    pub fn name_resolution_error(name: &str, kind: NumLocationKind) -> Self {
        Self {
            span: None,
            stage: ErrorStage::NameResolving,
            message: Some(format!("{kind} {name} not found!")),
        }
    }

    pub fn local_resolution_error(name: &str) -> Self {
        Self {
            span: None,
            stage: ErrorStage::NameResolving,
            message: Some(format!("Local {name} not found!")),
        }
    }

    pub fn label_resolution_error(name: &str) -> Self {
        Self {
            span: None,
            stage: ErrorStage::NameResolving,
            message: Some(format!("Label {name} not found in flow of block!")),
        }
    }

    pub fn type_error(expected: &SerializableWatType, actual: &SerializableWatType) -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!("Expected {expected} type but got {actual} type!")),
        }
    }

    pub fn setting_immutable_global_error(name: &str) -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!("Cannot set immutable Global {name}!")),
        }
    }

    pub fn no_instruction_provided(expected_type: &str) -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!(
                "Expected {expected_type} instruction but got nothing!"
            )),
        }
    }

    pub fn non_initializer_expression() -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some("Expect a single const expression for initalizing".to_string()),
        }
    }

    pub fn empty_stack(expected: usize) -> Self {
        Self::not_enough_on_stack(expected, 0)
    }
    pub fn not_enough_on_stack(expected: usize, actual: usize) -> Self {
        assert!(actual < expected);
        match (expected, actual) {
            (1, 0) => Self {
                span: None,
                stage: ErrorStage::TypeChecking,
                message: Some("Expected at least a value on the stack, but nothing is on the stack!".to_string()),
            },
            (_, 0) => Self {
                span: None,
                stage: ErrorStage::TypeChecking,
                message: Some(format!("Expected at least {expected} values on the stack, but nothing is on the stack!")),
            },
            _ =>  Self {
                span: None,
                stage: ErrorStage::TypeChecking,
                message: Some(format!("Expected at least {expected} values on the stack, but stack only has {actual}!")),
            },
        }
    }

    pub fn mismatched_inout(
        expected: &[SerializableWatType],
        actual: &[SerializableWatType],
        is_return: bool,
    ) -> Self {
        let expected = expected
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let actual = actual
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!(
                "Expected {} types to be [{expected}] on the stack, but stack has [{actual}]!",
                if is_return { "Return" } else { "Parameter" }
            )),
        }
    }

    pub fn duplicate_name_error(name: &str) -> Self {
        Self {
            span: None,
            stage: ErrorStage::NameResolving,
            message: Some(format!("Name {name} is defined multiple times")),
        }
    }

    pub fn unexpected_type(expected: &SerializableWatType, actual: &SerializableWatType) -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!(
                "Mismatched types, expected {expected}, but got {actual}."
            )),
        }
    }

    pub fn else_without_if_error() -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some("An else block should only follow after an if block.".to_string()),
        }
    }

    pub fn index_out_of_range_range(expected: usize, actual: usize) -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!("Index {actual} out of range: max {expected}.")),
        }
    }

    pub fn wrong_arity_error(expected: usize, actual: usize) -> Self {
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!(
                "Expect stack arity to be {expected}, but got {actual}."
            )),
        }
    }

    pub fn extra_items_on_stack_error(values: &[SerializableWatType]) -> Self {
        let found = values
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Self {
            span: None,
            stage: ErrorStage::TypeChecking,
            message: Some(format!("Expect stack to be empty, but found: {found}.")),
        }
    }
}
