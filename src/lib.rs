mod am;
pub mod spirv;
mod vm;

#[doc(inline)]
pub use vm::interpret;

#[doc(inline)]
pub use am::*;

pub mod wasm {
    pub use parity_wasm::elements::*;
    pub use parity_wasm::{deserialize_buffer, deserialize_file, serialize, serialize_to_file};
}

pub trait Main {
    fn main(&self) -> Option<u32>;
}
impl Main for wasm::Module {
    fn main(&self) -> Option<u32> {
        let main = self
            .export_section()?
            .entries()
            .iter()
            .find(|x| x.field() == "main")?;
        if let wasm::Internal::Function(m) = main.internal() {
            Some(*m)
        } else {
            None
        }
    }
}
