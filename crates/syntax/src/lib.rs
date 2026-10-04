pub mod did;
pub mod handle;
mod identifier;
pub mod nsid;

pub use identifier::Identifier;

mod private {
    pub trait Sealed {}
}
