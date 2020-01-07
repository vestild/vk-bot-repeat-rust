use simple_error::SimpleError;

pub type Error = simple_error::SimpleError;
pub type SimpleResult<T> = simple_error::SimpleResult<T>;

pub trait IntoSimpleError {
    fn wrap(&self, message: &str) -> SimpleError;
}

impl<T> IntoSimpleError for T where T : std::error::Error {
    fn wrap(&self, message: &str) -> Error {
        new(message, self)
    }
}

fn new<T: std::fmt::Display>(message: &str, cause: &T) -> Error {
    SimpleError::new(format!("{}, {}", message, cause))
}

pub trait IntoSimpleResult<T> {
    fn wrap_err(self, message: &str) -> SimpleResult<T>;
}

impl<T> IntoSimpleResult<T> for tokio::io::Result<T> {
    fn wrap_err(self, message: &str) -> SimpleResult<T> {
        match self {
            Ok(r) =>  Ok(r),
            Err(e) => Err(new(message, &e)),
        }
    }
}