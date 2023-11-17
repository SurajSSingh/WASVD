//! This module focuses on errors for WAT conversion.
use std::{fmt::Display, ops::Range};

use serde::{Deserialize, Serialize};
use specta::Type;

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
}
