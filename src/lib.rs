#![no_std]
extern crate alloc;

pub mod computing;
pub mod rendering;

#[doc(inline)]
pub use {
    computing::SerializedProgram,
    computing::PathStep,
    computing::serialize,
    rendering::NaiveRenderer,
};
