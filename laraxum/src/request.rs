/// Request methods.
pub mod method {
    /// GET
    pub struct Get;
    /// POST
    pub struct Create;
    /// PUT/PATCH
    pub struct Update;
    /// DELETE
    pub struct Delete;
}

/// Validate a request for a given request method.
pub trait Request<RequestType> {
    /// Request validation error.
    type Error;
    /// Validate a request.
    ///
    /// # Errors
    /// - validation fails.
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Build an error. For internal use.
pub fn error_builder<T, E>(result: &mut Result<T, E>, f: impl FnOnce(&mut E))
where
    E: Default,
{
    match *result {
        Ok(_) => {
            let mut e = E::default();
            f(&mut e);
            *result = Err(e);
        }
        Err(ref mut e) => {
            f(e);
        }
    }
}
