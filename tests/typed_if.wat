;; 12 12 12 12 45 45 45
;; Returns 12 if thread_id <= 3, else 45
(module
  (import "spv" "id" (global $id i32))
  (import "spv" "buffer:0:0:load" (func $load0 (param i32) (result i32)))
  (import "spv" "buffer:0:0:store" (func $store0 (param i32 i32)))

  (func $main
    (local $ptr i32)
    (set_local $ptr
      (i32.mul
        (get_global $id)
        (i32.const 4)))
    (call $store0
      (get_local $ptr)
      (if (result i32) (i32.le_u (get_global $id) (i32.const 3))
        (then (i32.const 12))
        (else (i32.const 45)))
      )
    )

  (start $main))
