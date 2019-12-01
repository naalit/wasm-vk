# This is a very simple script to convert some rustc-generated WAT into WAT that will work with wasm-vk.
# It uses standard input and output, so you can e.g. run `cat something.wat | python fix_wat.py >> something_else.wat`.
# It probably won't work with a new Rust program, you'll need to mess with it to get it to work.
# As a general guide, though, use no_std, and include this at the top of your script:
# ```rust
# #[link(wasm_import_module="spv")]
# extern {
#     fn trap() -> !;
#     fn id() -> usize;
#     fn sqrt(_: f32) -> f32; // This will be transformed into f32.sqrt in WASM
# }
#
# /// Get this thread's slot in the buffer
# fn thread_id() -> &'static mut u32 {
#     unsafe {
#         core::mem::transmute(id() * 4)
#     }
# }
# ```
# We emulate linear memory for loads and stores, but make sure your Rust code doesn't use more that about 64 bytes of it

i = ""
while True:
    try:
        i += input() + "\n"
    except EOFError:
        break

i = i.replace("call $id", "get_global $id")
i = i.replace('import "spv" "id" (func $id (type $t0))', 'import "spv" "id" (global $id i32)')
i = i.replace('(import "spv" "sqrt" (func $sqrt (type $t1)))', '')
i = i.replace('call $sqrt', 'f32.sqrt')
i = i.replace('func $main (export "main") (type $t4) (param $p0 i32) (param $p1 i32) (result i32)', 'func $main (export "main")')
i = i.replace('func $main (export "main") (type $t3) (param $p0 i32) (param $p1 i32) (result i32)', 'func $main (export "main")')
i = i.replace('func $main (export "main") (type $t2) (param $p0 i32) (param $p1 i32) (result i32)', 'func $main (export "main")')
i = i.replace('func $main (export "main") (type $t1) (param $p0 i32) (param $p1 i32) (result i32)', 'func $main (export "main")')
i = i.replace('(module\n', '(module\n  (start $main)\n')
# This might have some false positives
i = i.replace('(i32.const 0))\n', ')\n')

print(i)
