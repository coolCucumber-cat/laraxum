pub mod method {
    pub struct Get;
    pub struct Create;
    pub struct Update;
    pub struct Delete;
}

pub trait Request<RequestType> {
    type Error;
    /// Validate a request.
    ///
    /// # Errors
    /// - validation fails.
    fn validate(&self) -> Result<(), Self::Error>;
}

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
