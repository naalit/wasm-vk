# This is a very simple script to convert some rustc-generated WAT into WAT that will work with wasm-vk.
# It uses standard input and output, so you can e.g. run `cat something.wat | python fix_wat.py >> something_else.wat`.
# As a general guide, use no_std, and include this at the top of your script:
# ```rust
# #[link(wasm_import_module="spv")]
# extern {
#     fn trap() -> !;
#     // This thread's id, for use as a buffer index
#     fn id() -> usize;
#     fn sqrt(_: f32) -> f32; // This will be transformed into f32.sqrt in WASM
#
#     #[link_name="buffer:0:0:load"]
#     fn buf_get(i: usize) -> u32;
#     #[link_name="buffer:0:0:store"]
#     fn buf_set(i: usize, x: u32);
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
