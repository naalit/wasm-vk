;; 1 2 3 4 5 6
(module
  (start $main)
  (import "spv" "id" (global $id i32))
  (memory 1)

  (func $inc
    (param $p i32)
    (result i32)
    (i32.add (local.get $p) (i32.const 1)))

  ;; Testing a void function
  ;; one.wat tests one with no parameters that returns a value
  (func $store
    (param $val i32)
    (i32.store
      (i32.mul
        (i32.const 4)
        (global.get $id))
      (local.get $val))
    )

  ;; Testing multiple parameters
  (func $pick_the_first_one
    (param $a i32)
    (param i32 i32 i32)
    (result i32)
    (local.get $a))

  ;; TODO support multiple return values
  ;; Of course, parity_wasm currently doesn't even support that, so it may be a ways off

  (func $main
    (local $num i32)

    (local.set $num (global.get $id))
    (local.set $num (call $inc (local.get $num)))
    (local.set $num (call $pick_the_first_one (local.get $num) (i32.const 100000) (i32.const 436) (i32.const 676)))
    (call $store (local.get $num))
  )
)
