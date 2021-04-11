use std::cell::RefCell;
use std::fmt::Debug;
use std::io;

use futures::Future;
use tokio::runtime::Runtime;

use super::error::CBError;
use crate::public::Response;

pub trait Adapter<T> {
    type Result;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<Response<T>, CBError>> + Send + 'static;
}

pub trait AdapterNew: Sized {
    type Error: Debug;
    fn new() -> Result<Self, Self::Error>;
}

pub struct Sync(RefCell<Runtime>);

impl AdapterNew for Sync {
    type Error = io::Error;
    fn new() -> Result<Self, Self::Error> {
        let runtime = Runtime::new()?;
        Ok(Sync(RefCell::new(runtime)))
    }
}

impl<T> Adapter<T> for Sync
where
    T: Send + 'static,
{
    type Result = Result<T, CBError>;
    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<Response<T>, CBError>> + Send + 'static,
    {
        match self.0.borrow_mut().block_on(f) {
            Ok(response) => Ok(response.data),
            Err(e) => Err(e),
        }
    }
}

pub struct ASync;

impl AdapterNew for ASync {
    type Error = ();
    fn new() -> Result<Self, Self::Error> {
        Ok(ASync)
    }
}

impl<T> Adapter<T> for ASync {
    type Result = Box<dyn Future<Output = Result<T, CBError>> + Send>;

    fn process<F>(&self, f: F) -> Self::Result
    where
        F: Future<Output = Result<Response<T>, CBError>> + Send + 'static,
    {
        Box::new(async move {
            let response = f.await;
            match response {
                Ok(response) => Ok(response.data),
                Err(e) => Err(e),
            }
        })
    }
}
