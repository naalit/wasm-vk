;; 0 676 27 512 219 63
(module
  (; wasm-vk uses our declared start function as the SPIR-V entry point ;)
  (start $main)
  (; We declare an imported global for getting our thread index ;)
  (import "spv" "id" (global $id i32))

  (; We have one buffer, at set=0 and binding=0, of type i32.
     We specify buffers by importing load and store functions ;)
  (import "spv" "buffer:0:0:store" (func $buf_store (param i32 i32)))
  (import "spv" "buffer:0:0:load" (func $buf_load (param i32) (result i32)))

  (func $slot (result i32)
      (i32.mul
        (i32.const 4)
        (global.get $id)
      )
  )

  (; 'main' is a start function, so it doesn't have any parameters or return anything ;)
  (func $main
    (local $ptr i32)
    (local $val i32)
    (; $id is an invocation index - we need a byte index, so we multiply by 4
       (since we're storing a 4-byte number) ;)
    (local.set $ptr
      (call $slot)
      )
    (local.set $val
      (call $buf_load (local.get $ptr)))
    (; If this spot has a 1 in it, start looping ;)
    (if (i32.eq (local.get $val) (i32.const 1))
      (then
        (loop $continue
          (; Add one and square it until it's bigger than thirty
             we should get 676 ;)
          (local.set $val
            (i32.add (local.get $val) (i32.const 1)))
          (local.set $val
            (i32.mul (local.get $val) (local.get $val)))
          (br_if $continue (i32.le_u (local.get $val) (i32.const 30)))
        )
        (; Store 676 to the buffer and exit ;)
        (call $buf_store (local.get $ptr) (local.get $val))
        (return)
        ))
    (; If this spot has a 4 in it, change it to an 18,
       so the final result should be `(18 * 12) + 3` = 219 ;)
    (if (i32.eq (local.get $val) (i32.const 4))
      (then (local.set $val (i32.const 18))))
    (; If this spot has a 3 in it, return 512 ;)
    (if (i32.eq (local.get $val) (i32.const 3))
      (then (call $buf_store (local.get $ptr) (i32.const 512)))
      (else
        (; If this spot has a 0 in it, skip the updating logic - leave it at 0 ;)
        (if (i32.eq (local.get $val) (i32.const 0))
          (then (br 1))) (; Note that we branch out of the enclosing else block ;)
        (call $buf_store
          (local.get $ptr)
          (; Essentially ` *ptr = (*ptr * 12) + 3 ` ;)
          (i32.add
            (i32.mul
              (local.get $val)
              (i32.const 12))
            (i32.const 3)))
        ))
    ))
