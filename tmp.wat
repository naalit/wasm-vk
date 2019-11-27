(module
  (type $t0 (func (result i32)))
  (type $t1 (func (param f32) (result f32)))
  (type $t2 (func (param i32 i32)))
  (type $t3 (func (param i32 i32) (result i32)))
  (import "spv" "id" (func $id (type $t0)))
  (import "spv" "sqrt" (func $sqrt (type $t1)))
  (import "spv" "buffer:0:0:store" (func $buffer:0:0:store (type $t2)))
  (func $main (export "main") (type $t3) (param $p0 i32) (param $p1 i32) (result i32)
    (local $l2 f32) (local $l3 f32) (local $l4 f32) (local $l5 i32) (local $l6 f32) (local $l7 f32) (local $l8 f32) (local $l9 f32)
    (local.set $l2
      (f32.add
        (f32.mul
          (f32.convert_i32_u
            (i32.and
              (call $id)
              (i32.const 255)))
          (f32.const 0x1p-7 (;=0.0078125;)))
        (f32.const -0x1p+0 (;=-1;))))
    (local.set $l3
      (f32.add
        (f32.mul
          (f32.convert_i32_u
            (i32.shr_u
              (call $id)
              (i32.const 8)))
          (f32.const 0x1p-7 (;=0.0078125;)))
        (f32.const -0x1p+0 (;=-1;))))
    (local.set $l4
      (f32.const 0x0p+0 (;=0;)))
    (local.set $l5
      (i32.const 101))
    (local.set $l6
      (f32.const 0x0p+0 (;=0;)))
    (local.set $l7
      (f32.const 0x0p+0 (;=0;)))
    (block $B0
      (loop $L1
        (br_if $B0
          (i32.eqz
            (local.tee $l5
              (i32.add
                (local.get $l5)
                (i32.const -1)))))
        (block $B2
          (br_if $B2
            (f32.eq
              (local.tee $l9
                (f32.sqrt
                  (local.tee $l8
                    (f32.add
                      (f32.mul
                        (local.tee $l7
                          (f32.add
                            (local.get $l2)
                            (f32.sub
                              (f32.mul
                                (local.get $l7)
                                (local.get $l7))
                              (f32.mul
                                (local.get $l6)
                                (local.get $l6)))))
                        (local.get $l7))
                      (f32.mul
                        (local.tee $l6
                          (f32.add
                            (local.get $l3)
                            (f32.add
                              (local.tee $l6
                                (f32.mul
                                  (local.get $l6)
                                  (local.get $l7)))
                              (local.get $l6))))
                        (local.get $l6))))))
              (local.get $l9)))
          (local.set $l9
            (call $sqrt
              (local.get $l8))))
        (local.set $l4
          (f32.add
            (local.get $l4)
            (f32.const 0x1.47ae14p-7 (;=0.01;))))
        (br_if $L1
          (i32.xor
            (f32.gt
              (local.get $l9)
              (f32.const 0x1p+2 (;=4;)))
            (i32.const 1)))))
    (block $B3
      (block $B4
        (br_if $B4
          (i32.eqz
            (i32.and
              (f32.lt
                (local.tee $l6
                  (f32.mul
                    (local.get $l4)
                    (f32.const 0x1.fep+7 (;=255;))))
                (f32.const 0x1p+32 (;=4.29497e+09;)))
              (f32.ge
                (local.get $l6)
                (f32.const 0x0p+0 (;=0;))))))
        (local.set $l5
          (i32.trunc_f32_u
            (local.get $l6)))
        (br $B3))
      (local.set $l5
        (i32.const 0)))
    (call $buffer:0:0:store
      (i32.shl
        (call $id)
        (i32.const 2))
      (i32.or
        (i32.or
          (i32.or
            (i32.shl
              (local.get $l5)
              (i32.const 8))
            (local.get $l5))
          (i32.shl
            (local.get $l5)
            (i32.const 16)))
        (i32.const -16777216)))
    (i32.const 0))
  (table $T0 1 1 funcref)
  (memory $memory (export "memory") 16)
  (global $__data_end (export "__data_end") i32 (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576)))
  
