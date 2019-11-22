pub mod ir;
pub mod spirv;

pub mod wasm {
    pub use parity_wasm::elements::*;
    pub use parity_wasm::{deserialize_buffer, deserialize_file, serialize, serialize_to_file};
}

pub use wasm::IndexMap;
