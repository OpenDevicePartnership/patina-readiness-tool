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

    /// Indicates a failure in deserializing the JSON file. The `String`
    /// contains the filename that failed to deserialize.
    JSONSerializationFailed(String),

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
            ValidationAppError::JSONSerializationFailed(reason) => {
                write!(f, "Failed to serialize/deserialize JSON: {}", reason)
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
