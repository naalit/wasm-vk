mod am;
pub mod spirv;
mod vm;

#[doc(inline)]
pub use vm::interpret;

#[doc(inline)]
pub use am::*;

pub mod wasm {
    pub use parity_wasm::{deserialize_buffer, deserialize_file, serialize, serialize_to_file};
    pub use parity_wasm::elements::*;
}
