use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use validator::{Validate, ValidationError, ValidationErrors};
use warp::http::StatusCode;

#[derive(Debug, Serialize)]
pub struct Errors {
    errors: ValidationErrors,
}

pub type FieldName = &'static str;
pub type FieldErrorCode = &'static str;

impl Errors {
    pub fn new(errs: &[(FieldName, FieldErrorCode)]) -> Self {
        let mut errors = ValidationErrors::new();
        for (field, code) in errs {
            errors.add(field, ValidationError::new(code));
        }
        Self { errors }
    }

    fn get_json(self) -> serde_json::Value {
        let errors = self
            .errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                let codes = errors.into_iter().map(|err| err.code.to_string()).collect();
                (field, codes)
            })
            .collect::<HashMap<_, Vec<_>>>();

        json!({ "errors": errors })
    }
    pub fn respond_to(self) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
        let jsonerr = self.get_json();
        Ok(warp::reply::with_status(
            warp::reply::json(&jsonerr),
            StatusCode::BAD_REQUEST,
        ))
    }
    /*
    pub fn respond_without_status(self) -> Result<impl warp::Reply, Infallible> {
        let jsonerr=self.get_json();
        Ok(warp::reply::json(&jsonerr))
    }*/
}

pub struct FieldValidator {
    errors: ValidationErrors,
}

impl Default for FieldValidator {
    fn default() -> Self {
        Self {
            errors: ValidationErrors::new(),
        }
    }
}

impl FieldValidator {
    pub fn validate<T: Validate>(model: &T) -> Self {
        Self {
            errors: model.validate().err().unwrap_or_else(ValidationErrors::new),
        }
    }

    /// Convenience method to trigger early returns with ? operator.
    pub fn check(self) -> Result<(), Errors> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(Errors {
                errors: self.errors,
            })
        }
    }

    pub fn extract<T>(&mut self, field_name: &'static str, field: Option<T>) -> T
    where
        T: Default,
    {
        field.unwrap_or_else(|| {
            self.errors
                .add(field_name, ValidationError::new("can't be blank"));
            T::default()
        })
    }
}
