use crate::validate::ValidationResult;

/// A trait representing a generic validator that performs checks and returns
/// validation results.
///
/// Types implementing this trait are expected to define domain-specific
/// validation logic and return the outcome as a [`ValidationResult`] object.
pub trait Validator {
    /// Executes the validation logic and returns a [`ValidationResult`] object.
    fn validate(&self) -> ValidationResult;
}
