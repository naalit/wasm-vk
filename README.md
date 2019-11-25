# wasm-vk
`wasm-vk` is a command-line tool and Rust library to transpile WebAssembly into Vulkan SPIR-V.
It uses [`parity-wasm`](https://crates.io/crates/parity-wasm) to parse WASM and represent it internally,
and [`rspirv`](https://crates.io/crates/rspirv) to emit SPIR-V.
It makes no attempt to produce *optimized* SPIR-V - using spirv-opt after is probably a good idea.

# Why
WebAssembly was never meant just for the Web, it's meant to be used on many different platforms.
To that end, it doesn't have native support for most things, and requires an *embedder*.
It's also very simple, and only relies on functionality that's very well supported across architectures.

Because of all this, we can create a WebAssembly embedder that runs on the GPU, and we can support almost all of WebAssembly.

# Mapping
`wasm-vk` compiles a WebAssembly module into a Vulkan compute shader, currently with local size hardcoded as 64x1x1.
WASM's *linear memory* is represented as a single Vulkan buffer, at descriptor set 0 and binding 0, which can be both read and written to by the shader.
It uses the module's start function as the entry point, and shaders can define a global i32 "spv.id" which represents the thread index (gl_GlobalInvocationID.x, specifically).
See `examples/comp.wat` for an example of a compute shader written in WebAssembly, or `examples/image.wat` for one written in Rust and compiled to WebAssembly.

We'll eventually add imports for other SPIR-V builtins, and we may use the atomic operations from the WebAssembly threads proposal, and probably force the memory to be marked `shared`.

# Usage
### Command-line usage
```
wasm-vk [options] <input.wasm> [output.spv]

If no output file is given, it will default to 'out.spv'.

Options:
-v, --verbose       Show more output, including dissasembled SPIR-V
-h, --help          Show this help
```

### Library usage
wasm-vk isn't on crates.io yet, but you can try it if you want.
You can use `wasm_vk::ir::to_base()`, which you can pass a `parity-wasm` `Module`, to create a vector of functions in the `Base` IR.
We re-export `deserialize` and `deserialize_file` from `parity-wasm`, but you can also create a `parity-wasm` `Module` yourself.
Then you can create a `wasm_vk::spirv::Ctx` and pass each function to `Ctx::fun()`.
When you're done, pass the `Module::start_section()` of the original Wasm `Module` to `Ctx::finish()` to get a `rspirv::dr::Module`, which you can pass to `wasm_vk::spirv::module_bytes` if you need the bytes to pass to Vulkan.

Note that wasm-vk doesn't interact at all with Vulkan - it just produces SPIR-V bytes for use with any Vulkan library.
See `examples/vulkano.rs` for an example using [`Vulkano`](https://crates.io/crates/vulkano) to load and run a WebAssembly compute shader.
See `examples/image.rs` for an example of generating an image in a compute shader.

# Current status
See `examples/comp.wat` for most of what `wasm-vk` currently supports.
Supported instructions:
```
General operations:
- nop
- i32.load
- i32.store
- global.get (just for 'spv.id' builtin)
- local.set
- local.get
- local.tee
Numeric operations: All i32 and f32 instructions EXCEPT:
- i32.clz
- i32.ctz
- i32.popcnt
- i32.rem_*
- i32.rotr
- i32.rotl
- f32.trunc
- f32.nearest
- f32.copysign
- reinterpret instructions
Control flow (note: we don't currently support blocks returning things):
- loop
- block
- if/then/else
- br
- br_if
- return (without a value)
- call (we support functions in general)
```
