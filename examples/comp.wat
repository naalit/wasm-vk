(module
  (; wasm-vk needs us to export the main function as "main" ;)
  (export "main" (func $main))
  (import "spv" "id" (global $id i32))
  (; This is the buffer we're allowed to read and write
     The size declared here doesn't matter, because wasm-vk needs to be passed a buffer
     that the user has allocated with whatever size they want ;)
  (memory $mem 1)
  (; The main function takes it's invocation index as a parameter and doesn't return anything;)
  (func $main
    (local $ptr i32)
    (local $val i32)
    (; $id is an invocation index - we need a byte index, so we multiply by 4
       (since we're storing a 4-byte number) ;)
    (local.set $ptr
      (i32.mul
        (i32.const 4)
        (global.get $id)))
    (local.set $val
      (i32.load (local.get $ptr)))
    (; If this spot has a 4 in it, change it to an 18,
       so the final result should be `(18 * 12) + 3` = 219 ;)
    (if (i32.eq (local.get $val) (i32.const 4))
      (then (local.set $val (i32.const 18))))
    (; If this spot has a 3 in it, return 512 ;)
    (if (i32.eq (local.get $val) (i32.const 3))
      (then (i32.store (local.get $ptr) (i32.const 512)))
      (else
        (if (i32.eq (local.get $val) (i32.const 0))
          (then (br 1)))
        (i32.store
          (local.get $ptr)
          (; Essentially ` *ptr = (*ptr * 12) + 3 ` ;)
          (i32.add
            (i32.mul
              (local.get $val)
              (i32.const 12))
            (i32.const 3)))
        ))
    ))
    
