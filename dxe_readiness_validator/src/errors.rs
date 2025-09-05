//! Error types used throughout the DXE readiness validator application.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
use std::fmt;

/// Represents possible errors that can occur during the validation app's
/// execution.
#[derive(Debug, PartialEq)]
pub enum ValidationAppError {
    /// Indicates that the command-line arguments are invalid. The `String`
    /// contains the missing or incorrect argument.
    InvalidCommandLine(String),

    /// Indicates that the specified JSON file could not be found or opened. The
    /// `String` contains the filename that failed to be read.
    JSONFileNotFound(String),

    /// Indicates that the specified JSON file contain invalid 'utf-8' byte
    /// sequence. The `String` contains the filename that failed to be read. The
    /// other `String` contains the error message.
    JSONFileContentError(String, String),

    /// Indicates a failure in deserializing the JSON file. The `String`
    /// contains the filename that failed to deserialize. The other `String`
    /// contains the error message.
    JSONSerializationFailed(String, String),

    /// Indicates that the parsed data contains an empty HOB list.
    EmptyHobList,

    /// Indicates that the parsed data contains an empty Firmware Volume list.
    EmptyFvList,

    /// Indicates that one or more validation rules were violated. The `u32` is
    /// the number of violations/errors detected.
    ValidationErrors(u32),
}

impl fmt::Display for ValidationAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationAppError::InvalidCommandLine(argument) => {
                write!(f, "Invalid command line argument: {}", argument)
            }
            ValidationAppError::JSONFileNotFound(path) => {
                write!(f, "JSON file not found: {}", path)
            }
            ValidationAppError::JSONFileContentError(path, err) => {
                write!(f, "Error reading the JSON file {} contents. Error: {}", path, err)
            }
            ValidationAppError::JSONSerializationFailed(path, err) => {
                write!(f, "Failed to serialize/deserialize JSON: {}. Error: {}", path, err)
            }
            ValidationAppError::EmptyHobList => {
                write!(f, "The HOB list is empty.")
            }
            ValidationAppError::EmptyFvList => {
                write!(f, "The FV list is empty.")
            }
            ValidationAppError::ValidationErrors(violations) => {
                write!(f, "Found {} validation errors", violations)
            }
        }
    }
}
