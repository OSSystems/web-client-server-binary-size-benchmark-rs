pub trait LocalClientImpl: Sized {
    fn new() -> Self;
}

pub trait RemoteClientImpl: Sized {
    fn new() -> Self;
}

pub trait ServerImpl: Sized {
    fn new() -> Self;
    fn run(self);
}

pub mod prelude {
    pub use super::{LocalClientImpl, RemoteClientImpl, ServerImpl};
}

#[cfg(feature = "dummy")]
pub mod dummy;
