;; 0 1 2 3 4 5
(module
  (start $main)
  (type $t0 (func (result i32)))
  (type $t1 (func (param i32) (result i32)))
  (type $t2 (func (param i32 i32)))
  (type $t3 (func (param i32 i32) (result i32)))
  (import "spv" "id" (global $id i32))
  (import "spv" "buffer:0:0:load" (func $buffer:0:0:load (type $t1)))
  (import "spv" "buffer:0:0:store" (func $buffer:0:0:store (type $t2)))
  (func $playground::return_struct::hc07a7084df86c0ca (type $t2) (param $p0 i32) (param $p1 i32)
    (i32.store offset=4
      (local.get $p0)
      (i32.load
        (local.get $p1)))
    (i32.store
      (local.get $p0)
      (i32.load
        (i32.add
          (local.get $p1)
          (i32.const 4)))))
  (func $main (export "main")
    (local $l2 i32) (local $l3 i32) (local $l4 i32)
    (global.set $g0
      (local.tee $l2
        (i32.sub
          (global.get $g0)
          (i32.const 16))))
    (i32.store offset=12
      (local.get $l2)
      (call $buffer:0:0:load
        (i32.shl
          (get_global $id)
          (i32.const 2))))
    (call $playground::return_struct::hc07a7084df86c0ca
      (local.get $l2)
      (i32.add
        (local.get $l2)
        (i32.const 12)))
    (local.set $l3
      (i32.load offset=4
        (local.get $l2)))
    (local.set $l4
      (i32.load
        (local.get $l2)))
    (call $buffer:0:0:store
      (i32.shl
        (get_global $id)
        (i32.const 2))
      (i32.add
        (local.get $l4)
        (local.get $l3)))
    (global.set $g0
      (i32.add
        (local.get $l2)
        (i32.const 16)))
    )
  (table $T0 1 1 funcref)
  (memory $memory (export "memory") 16)
  (global $g0 (mut i32) (i32.const 1048576))
  (global $__data_end (export "__data_end") i32 (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576)))
