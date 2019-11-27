;; Adds the values from the two buffers together and stores them in the first
(module
  (import "spv" "id" (global $id i32))
  (import "spv" "buffer:0:0:load" (func $load0 (param i32) (result i32)))
  (import "spv" "buffer:0:0:store" (func $store0 (param i32 i32)))
  (import "spv" "buffer:0:1:load" (func $load1 (param i32) (result i32)))

  (func $main
    (local $ptr i32)
    (set_local $ptr
      (i32.mul
        (get_global $id)
        (i32.const 4)))
    (call $store0
      (get_local $ptr)
      (i32.add
        (call $load0 (get_local $ptr))
        (call $load1 (get_local $ptr))))
    )

  (start $main))
