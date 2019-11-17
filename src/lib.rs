mod am;
pub mod spirv;
mod vm;
pub mod ir;
pub mod sr;

#[doc(inline)]
pub use vm::interpret;

#[doc(inline)]
pub use am::*;

pub mod wasm {
    pub use parity_wasm::elements::*;
    pub use parity_wasm::{deserialize_buffer, deserialize_file, serialize, serialize_to_file};
}
