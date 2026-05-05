use anyhow::Result;
use std::ffi::OsString;

/// A trait for generating requests dynamically.
pub trait RequestGenerator: Send + Sync {
    /// Generates a request for the given iteration.
    ///
    /// # Returns
    /// A tuple containing the `reqwest::Request` and an optional base file name
    /// for saving the response body.
    fn generate(&self, iteration: u64) -> Result<(reqwest::Request, Option<OsString>)>;
}

impl<F> RequestGenerator for F
where
    F: Fn(u64) -> Result<(reqwest::Request, Option<OsString>)> + Send + Sync,
{
    fn generate(&self, iteration: u64) -> Result<(reqwest::Request, Option<OsString>)> {
        self(iteration)
    }
}
