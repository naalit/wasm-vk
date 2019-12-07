pub mod ir;
pub mod spirv;

pub mod wasm {
    pub use parity_wasm::elements::*;
    pub use parity_wasm::{deserialize_buffer, deserialize_file, serialize, serialize_to_file};

    pub fn block_ty_to_option(b: BlockType) -> Option<ValueType> {
        match b {
            BlockType::Value(v) => Some(v),
            BlockType::NoResult => None,
        }
    }
}

pub use wasm::IndexMap;
