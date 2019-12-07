;; This is an example both of rendering an image in a compute shader, and of compiling a somewhat less trivial Rust program.
;; It's a simple raymarcher that renders two colored spheres.
;; The Rust source is at the bottom - I just used the playground to compile it to WASM, and the fix_wat.py script.
;; Make sure to have the playground compile in release mode - it's much smaller.
(module
  (start $main)
  (type $t0 (func (result i32)))
  (type $t1 (func (param f32) (result f32)))
  (type $t2 (func (param i32 i32)))
  (type $t3 (func (param i32 i32) (result i32)))
  (type $t4 (func (param f32 f32) (result f32)))
  (import "spv" "id" (global $id i32))

  (import "spv" "buffer:0:0:store" (func $buffer:0:0:store (type $t2)))
  (func $main (export "main")
    (local $l2 f32) (local $l3 i32) (local $l4 f32) (local $l5 f32) (local $l6 f32) (local $l7 f32) (local $l8 f32) (local $l9 f32) (local $l10 f32) (local $l11 f32) (local $l12 i32) (local $l13 f32) (local $l14 f32) (local $l15 f32) (local $l16 f32) (local $l17 f32) (local $l18 f32) (local $l19 i32)
    (local.set $l2
      (f32.const 0x0p+0 (;=0;)))
    (block $B0
      (br_if $B0
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l7
                (f32.add
                  (f32.mul
                    (local.tee $l8
                      (f32.add
                        (local.tee $l5
                          (f32.mul
                            (local.tee $l4
                              (f32.add
                                (f32.mul
                                  (f32.convert_i32_u
                                    (i32.and
                                      (local.tee $l3
                                        (get_global $id))
                                      (i32.const 1023)))
                                  (f32.const 0x1p-9 (;=0.00195312;)))
                                (f32.const -0x1p+0 (;=-1;))))
                            (f32.const 0x0p+0 (;=0;))))
                        (f32.add
                          (local.tee $l7
                            (f32.mul
                              (local.tee $l6
                                (f32.sub
                                  (f32.const 0x1p+0 (;=1;))
                                  (f32.mul
                                    (f32.convert_i32_u
                                      (i32.shr_u
                                        (local.get $l3)
                                        (i32.const 10)))
                                    (f32.const 0x1p-9 (;=0.00195312;)))))
                              (f32.const 0x0p+0 (;=0;))))
                          (f32.const 0x1p+0 (;=1;)))))
                    (local.get $l8))
                  (f32.add
                    (f32.mul
                      (local.tee $l5
                        (f32.add
                          (f32.add
                            (local.get $l6)
                            (f32.const 0x0p+0 (;=0;)))
                          (local.get $l5)))
                      (local.get $l5))
                    (f32.mul
                      (local.tee $l6
                        (f32.add
                          (local.get $l4)
                          (f32.add
                            (local.get $l7)
                            (f32.const 0x0p+0 (;=0;)))))
                      (local.get $l6)))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l7))))
    (local.set $l9
      (f32.div
        (local.get $l8)
        (local.get $l4)))
    (local.set $l10
      (f32.div
        (local.get $l5)
        (local.get $l4)))
    (local.set $l11
      (f32.div
        (local.get $l6)
        (local.get $l4)))
    (local.set $l4
      (f32.const -0x1p+1 (;=-2;)))
    (local.set $l12
      (i32.const 64))
    (local.set $l6
      (f32.const 0x0p+0 (;=0;)))
    (local.set $l8
      (f32.const 0x0p+0 (;=0;)))
    (block $B1
      (loop $L2
        (block $B3
          (br_if $B3
            (f32.eq
              (local.tee $l5
                (f32.sqrt
                  (local.tee $l13
                    (f32.add
                      (f32.mul
                        (local.get $l4)
                        (local.get $l4))
                      (f32.add
                        (local.tee $l7
                          (f32.mul
                            (local.get $l6)
                            (local.get $l6)))
                        (f32.mul
                          (local.get $l8)
                          (local.get $l8)))))))
              (local.get $l5)))
          (local.set $l5
            (f32.sqrt
              (local.get $l13))))
        (local.set $l13
          (f32.add
            (local.get $l5)
            (f32.const -0x1p+0 (;=-1;))))
        (block $B4
          (br_if $B4
            (f32.eq
              (local.tee $l5
                (f32.sqrt
                  (local.tee $l7
                    (f32.add
                      (f32.mul
                        (local.tee $l5
                          (f32.add
                            (local.get $l4)
                            (f32.const -0x1p+0 (;=-1;))))
                        (local.get $l5))
                      (f32.add
                        (local.get $l7)
                        (f32.mul
                          (local.tee $l5
                            (f32.add
                              (local.get $l8)
                              (f32.const 0x1p+1 (;=2;))))
                          (local.get $l5)))))))
              (local.get $l5)))
          (local.set $l5
            (f32.sqrt
              (local.get $l7))))
        (br_if $B1
          (f32.lt
            (local.tee $l5
              (call $fminf
                (local.get $l13)
                (f32.add
                  (local.get $l5)
                  (f32.const -0x1p+0 (;=-1;)))))
            (f32.const 0x1.47ae14p-7 (;=0.01;))))
        (local.set $l2
          (f32.add
            (local.get $l2)
            (local.get $l5)))
        (local.set $l4
          (f32.add
            (local.get $l4)
            (f32.mul
              (local.get $l9)
              (local.get $l5))))
        (local.set $l6
          (f32.add
            (local.get $l6)
            (f32.mul
              (local.get $l10)
              (local.get $l5))))
        (local.set $l8
          (f32.add
            (local.get $l8)
            (f32.mul
              (local.get $l11)
              (local.get $l5))))
        (br_if $L2
          (local.tee $l12
            (i32.add
              (local.get $l12)
              (i32.const -1)))))
      (local.set $l2
        (f32.const 0x0p+0 (;=0;))))
    (block $B5
      (br_if $B5
        (f32.eq
          (local.tee $l13
            (f32.sqrt
              (local.tee $l2
                (f32.add
                  (local.tee $l5
                    (f32.mul
                      (local.tee $l8
                        (f32.add
                          (f32.mul
                            (local.get $l9)
                            (local.get $l2))
                          (f32.const -0x1p+1 (;=-2;))))
                      (local.get $l8)))
                  (local.tee $l14
                    (f32.add
                      (local.tee $l11
                        (f32.mul
                          (local.tee $l6
                            (f32.add
                              (f32.mul
                                (local.get $l11)
                                (local.get $l2))
                              (f32.const 0x0p+0 (;=0;))))
                          (local.get $l6)))
                      (local.tee $l4
                        (f32.mul
                          (local.tee $l7
                            (f32.add
                              (f32.mul
                                (local.get $l10)
                                (local.get $l2))
                              (f32.const 0x0p+0 (;=0;))))
                          (local.get $l7)))))))))
          (local.get $l13)))
      (local.set $l13
        (f32.sqrt
          (local.get $l2))))
    (block $B6
      (br_if $B6
        (f32.eq
          (local.tee $l9
            (f32.sqrt
              (local.tee $l10
                (f32.add
                  (local.tee $l2
                    (f32.mul
                      (local.tee $l2
                        (f32.add
                          (local.get $l8)
                          (f32.const -0x1p+0 (;=-1;))))
                      (local.get $l2)))
                  (local.tee $l16
                    (f32.add
                      (local.get $l4)
                      (local.tee $l15
                        (f32.mul
                          (local.tee $l9
                            (f32.add
                              (local.get $l6)
                              (f32.const 0x1p+1 (;=2;))))
                          (local.get $l9)))))))))
          (local.get $l9)))
      (local.set $l9
        (f32.sqrt
          (local.get $l10))))
    (block $B7
      (br_if $B7
        (f32.eq
          (local.tee $l10
            (f32.sqrt
              (local.tee $l18
                (f32.add
                  (local.get $l5)
                  (f32.add
                    (local.get $l4)
                    (f32.mul
                      (local.tee $l17
                        (f32.add
                          (local.get $l6)
                          (f32.const 0x1.47ae14p-7 (;=0.01;))))
                      (local.get $l17)))))))
          (local.get $l10)))
      (local.set $l10
        (f32.sqrt
          (local.get $l18))))
    (local.set $l18
      (f32.add
        (local.get $l10)
        (f32.const -0x1p+0 (;=-1;))))
    (block $B8
      (br_if $B8
        (f32.eq
          (local.tee $l10
            (f32.sqrt
              (local.tee $l17
                (f32.add
                  (local.get $l2)
                  (f32.add
                    (local.get $l4)
                    (f32.mul
                      (local.tee $l10
                        (f32.add
                          (local.get $l17)
                          (f32.const 0x1p+1 (;=2;))))
                      (local.get $l10)))))))
          (local.get $l10)))
      (local.set $l10
        (f32.sqrt
          (local.get $l17))))
    (local.set $l17
      (call $fminf
        (local.get $l18)
        (f32.add
          (local.get $l10)
          (f32.const -0x1p+0 (;=-1;)))))
    (block $B9
      (br_if $B9
        (f32.eq
          (local.tee $l6
            (f32.sqrt
              (local.tee $l18
                (f32.add
                  (local.get $l5)
                  (f32.add
                    (local.get $l4)
                    (f32.mul
                      (local.tee $l10
                        (f32.add
                          (local.get $l6)
                          (f32.const -0x1.47ae14p-7 (;=-0.01;))))
                      (local.get $l10)))))))
          (local.get $l6)))
      (local.set $l6
        (f32.sqrt
          (local.get $l18))))
    (local.set $l6
      (f32.add
        (local.get $l6)
        (f32.const -0x1p+0 (;=-1;))))
    (block $B10
      (br_if $B10
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l10
                (f32.add
                  (local.get $l2)
                  (f32.add
                    (local.get $l4)
                    (f32.mul
                      (local.tee $l10
                        (f32.add
                          (local.get $l10)
                          (f32.const 0x1p+1 (;=2;))))
                      (local.get $l10)))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l10))))
    (local.set $l10
      (call $fminf
        (local.get $l6)
        (f32.add
          (local.get $l4)
          (f32.const -0x1p+0 (;=-1;)))))
    (block $B11
      (br_if $B11
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l18
                (f32.add
                  (local.get $l5)
                  (f32.add
                    (local.get $l11)
                    (local.tee $l6
                      (f32.mul
                        (local.tee $l4
                          (f32.add
                            (local.get $l7)
                            (f32.const 0x1.47ae14p-7 (;=0.01;))))
                        (local.get $l4))))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l18))))
    (local.set $l18
      (f32.add
        (local.get $l4)
        (f32.const -0x1p+0 (;=-1;))))
    (block $B12
      (br_if $B12
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l6
                (f32.add
                  (local.get $l2)
                  (f32.add
                    (local.get $l15)
                    (local.get $l6))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l6))))
    (local.set $l18
      (call $fminf
        (local.get $l18)
        (f32.add
          (local.get $l4)
          (f32.const -0x1p+0 (;=-1;)))))
    (block $B13
      (br_if $B13
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l5
                (f32.add
                  (local.get $l5)
                  (f32.add
                    (local.get $l11)
                    (local.tee $l6
                      (f32.mul
                        (local.tee $l4
                          (f32.add
                            (local.get $l7)
                            (f32.const -0x1.47ae14p-7 (;=-0.01;))))
                        (local.get $l4))))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l5))))
    (local.set $l5
      (f32.add
        (local.get $l4)
        (f32.const -0x1p+0 (;=-1;))))
    (block $B14
      (br_if $B14
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l6
                (f32.add
                  (local.get $l2)
                  (f32.add
                    (local.get $l15)
                    (local.get $l6))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l6))))
    (local.set $l2
      (call $fminf
        (local.get $l5)
        (f32.add
          (local.get $l4)
          (f32.const -0x1p+0 (;=-1;)))))
    (block $B15
      (br_if $B15
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l6
                (f32.add
                  (local.get $l14)
                  (f32.mul
                    (local.tee $l5
                      (f32.add
                        (local.get $l8)
                        (f32.const 0x1.47ae14p-7 (;=0.01;))))
                    (local.get $l5))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l6))))
    (local.set $l6
      (f32.add
        (local.get $l4)
        (f32.const -0x1p+0 (;=-1;))))
    (block $B16
      (br_if $B16
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l5
                (f32.add
                  (local.get $l16)
                  (f32.mul
                    (local.tee $l4
                      (f32.add
                        (local.get $l5)
                        (f32.const -0x1p+0 (;=-1;))))
                    (local.get $l4))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l5))))
    (local.set $l5
      (f32.add
        (local.get $l13)
        (f32.const -0x1p+0 (;=-1;))))
    (local.set $l7
      (f32.add
        (local.get $l9)
        (f32.const -0x1p+0 (;=-1;))))
    (local.set $l13
      (call $fminf
        (local.get $l6)
        (f32.add
          (local.get $l4)
          (f32.const -0x1p+0 (;=-1;)))))
    (block $B17
      (br_if $B17
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l8
                (f32.add
                  (local.get $l14)
                  (f32.mul
                    (local.tee $l6
                      (f32.add
                        (local.get $l8)
                        (f32.const -0x1.47ae14p-7 (;=-0.01;))))
                    (local.get $l6))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l8))))
    (local.set $l12
      (f32.lt
        (local.get $l5)
        (local.get $l7)))
    (local.set $l8
      (f32.sub
        (local.get $l17)
        (local.get $l10)))
    (local.set $l5
      (f32.sub
        (local.get $l18)
        (local.get $l2)))
    (local.set $l2
      (f32.add
        (local.get $l4)
        (f32.const -0x1p+0 (;=-1;))))
    (block $B18
      (br_if $B18
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l6
                (f32.add
                  (local.get $l16)
                  (f32.mul
                    (local.tee $l4
                      (f32.add
                        (local.get $l6)
                        (f32.const -0x1p+0 (;=-1;))))
                    (local.get $l4))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l6))))
    (local.set $l6
      (select
        (f32.const 0x1p-1 (;=0.5;))
        (f32.const 0x1.666666p-1 (;=0.7;))
        (local.get $l12)))
    (block $B19
      (br_if $B19
        (f32.eq
          (local.tee $l4
            (f32.sqrt
              (local.tee $l7
                (f32.add
                  (f32.add
                    (f32.mul
                      (local.get $l8)
                      (local.get $l8))
                    (f32.mul
                      (local.get $l5)
                      (local.get $l5)))
                  (f32.mul
                    (local.tee $l2
                      (f32.sub
                        (local.get $l13)
                        (call $fminf
                          (local.get $l2)
                          (f32.add
                            (local.get $l4)
                            (f32.const -0x1p+0 (;=-1;))))))
                    (local.get $l2))))))
          (local.get $l4)))
      (local.set $l4
        (f32.sqrt
          (local.get $l7))))
    (local.set $l7
      (select
        (f32.const 0x1p+0 (;=1;))
        (f32.const 0x1.333334p-2 (;=0.3;))
        (local.get $l12)))
    (block $B20
      (block $B21
        (br_if $B21
          (i32.eqz
            (i32.and
              (f32.lt
                (local.tee $l8
                  (f32.mul
                    (call $fminf
                      (f32.add
                        (f32.mul
                          (local.get $l6)
                          (f32.const 0x1.99999ap-5 (;=0.05;)))
                        (f32.mul
                          (local.get $l6)
                          (local.tee $l4
                            (f32.add
                              (f32.add
                                (f32.div
                                  (local.get $l5)
                                  (local.get $l4))
                                (f32.mul
                                  (f32.div
                                    (local.get $l8)
                                    (local.get $l4))
                                  (f32.const 0x0p+0 (;=0;))))
                              (f32.mul
                                (f32.div
                                  (local.get $l2)
                                  (local.get $l4))
                                (f32.const 0x0p+0 (;=0;)))))))
                      (f32.const 0x1p+0 (;=1;)))
                    (f32.const 0x1.fep+7 (;=255;))))
                (f32.const 0x1p+32 (;=4.29497e+09;)))
              (f32.ge
                (local.get $l8)
                (f32.const 0x0p+0 (;=0;))))))
        (local.set $l19
          (i32.trunc_f32_u
            (local.get $l8)))
        (br $B20))
      (local.set $l19
        (i32.const 0)))
    (local.set $l8
      (select
        (f32.const 0x1.99999ap-3 (;=0.2;))
        (f32.const 0x1.99999ap-2 (;=0.4;))
        (local.get $l12)))
    (local.set $l12
      (i32.shl
        (local.get $l19)
        (i32.const 8)))
    (block $B22
      (block $B23
        (br_if $B23
          (i32.eqz
            (i32.and
              (f32.lt
                (local.tee $l5
                  (f32.mul
                    (call $fminf
                      (f32.add
                        (f32.mul
                          (local.get $l7)
                          (f32.const 0x1.99999ap-5 (;=0.05;)))
                        (f32.mul
                          (local.get $l7)
                          (local.get $l4)))
                      (f32.const 0x1p+0 (;=1;)))
                    (f32.const 0x1.fep+7 (;=255;))))
                (f32.const 0x1p+32 (;=4.29497e+09;)))
              (f32.ge
                (local.get $l5)
                (f32.const 0x0p+0 (;=0;))))))
        (local.set $l19
          (i32.trunc_f32_u
            (local.get $l5)))
        (br $B22))
      (local.set $l19
        (i32.const 0)))
    (local.set $l12
      (i32.or
        (local.get $l12)
        (local.get $l19)))
    (block $B24
      (block $B25
        (br_if $B25
          (i32.eqz
            (i32.and
              (f32.lt
                (local.tee $l4
                  (f32.mul
                    (call $fminf
                      (f32.add
                        (f32.mul
                          (local.get $l8)
                          (f32.const 0x1.99999ap-5 (;=0.05;)))
                        (f32.mul
                          (local.get $l8)
                          (local.get $l4)))
                      (f32.const 0x1p+0 (;=1;)))
                    (f32.const 0x1.fep+7 (;=255;))))
                (f32.const 0x1p+32 (;=4.29497e+09;)))
              (f32.ge
                (local.get $l4)
                (f32.const 0x0p+0 (;=0;))))))
        (local.set $l19
          (i32.trunc_f32_u
            (local.get $l4)))
        (br $B24))
      (local.set $l19
        (i32.const 0)))
    (call $buffer:0:0:store
      (i32.shl
        (local.get $l3)
        (i32.const 2))
      (i32.or
        (i32.or
          (local.get $l12)
          (i32.shl
            (local.get $l19)
            (i32.const 16)))
        (i32.const -16777216)))
    )
  (func $fminf (type $t4) (param $p0 f32) (param $p1 f32) (result f32)
    (select
      (local.get $p0)
      (select
        (local.get $p0)
        (local.get $p1)
        (f32.lt
          (local.get $p0)
          (local.get $p1)))
      (f32.ne
        (local.get $p1)
        (local.get $p1))))
  (table $T0 1 1 funcref)
  (memory $memory (export "memory") 16)
  (global $__data_end (export "__data_end") i32 (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576)))

;; Rust Source
(;
#![feature(start)]
#![no_std]

// SIZExSIZE
const SIZE: usize = 1024;

#[link(wasm_import_module="spv")]
extern {
    fn trap() -> !;
    fn id() -> usize;
    fn sqrt(_: f32) -> f32;

    #[link_name="buffer:0:0:load"]
    fn buf_get(i: usize) -> u32;
    #[link_name="buffer:0:0:store"]
    fn buf_set(i: usize, x: u32);
}

#[panic_handler]
fn handle_panic(_x: &core::panic::PanicInfo) -> ! {
    unsafe {
        trap()
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn length(self) -> f32 {
        let Vec3 { x, y, z } = self;
        unsafe {
            sqrt(x*x + y*y + z*z)
        }
    }

    fn normalize(self) -> Vec3 {
        let l = self.length();
        let Vec3 { x, y, z } = self;
        Vec3 {
            x: x / l,
            y: y / l,
            z: z / l,
        }
    }

    fn sx(self, x: f32) -> Vec3 {
        Vec3 {
            x,
            ..self
        }
    }

    fn sy(self, y: f32) -> Vec3 {
        Vec3 {
            y,
            ..self
        }
    }

    fn sz(self, z: f32) -> Vec3 {
        Vec3 {
            z,
            ..self
        }
    }

    fn dot(self, other: Vec3) -> f32 {
        self.x*other.x + self.y*other.y + self.z*other.z
    }

    const fn zero() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

use core::ops::*;
impl Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: f32) -> Vec3 {
        Vec3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}
impl Sub<f32> for Vec3 {
    type Output = Vec3;
    fn sub(self, other: f32) -> Vec3 {
        Vec3 {
            x: self.x - other,
            y: self.y - other,
            z: self.z - other,
        }
    }
}

fn rgb(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.min(1.0) * 255.0) as u32;
    let g = (g.min(1.0) * 255.0) as u32;
    let b = (b.min(1.0) * 255.0) as u32;
    255 << 24 // Alpha = 1.0
        | b << 16
        | g << 8
        | r
}

fn scene(p: Vec3) -> f32 {
    (p.length() - 1.0).min((p - Vec3 {
        z: 1.0,
        x: -2.0,
        y: 0.0,
    }).length() - 1.0)
}

fn color(p: Vec3) -> Vec3 {
    let a = p.length() - 1.0;
    let b = (p - Vec3 {
        z: 1.0,
        x: -2.0,
        y: 0.0,
    }).length() - 1.0;
    if a < b {
        Vec3 {
            x: 1.0,
            y: 0.5,
            z: 0.2,
        }
    } else {
        Vec3 {
            x: 0.3,
            y: 0.7,
            z: 0.4,
        }
    }
}

fn normal(p: Vec3) -> Vec3 {
    let e = 0.01;
    Vec3 {
        x: scene(p.sx(p.x + e)) - scene(p.sx(p.x - e)),
        y: scene(p.sy(p.y + e)) - scene(p.sy(p.y - e)),
        z: scene(p.sz(p.z + e)) - scene(p.sz(p.z - e)),
    }.normalize()
}

// Returns the t-value on a hit
fn trace(ro: Vec3, rd: Vec3) -> Option<f32> {
    let mut p = ro;
    let mut t = 0.0;

    for _ in 0..64 {
        let d = scene(p);
        if d < 0.01 {
            return Some(t);
        }
        t += d;
        p = p + rd * d;
    }

    None
}

fn shade(col: Vec3, l: Vec3, n: Vec3) -> Vec3 {
    let cos_theta = l.dot(n);

    col * cos_theta // Lambertian. No 1/pi term because we're assuming colors are already adjusted for that
        + col * 0.05 // Ambient
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    let id = unsafe { id() };
    let x = id % SIZE;
    let y = id / SIZE;

    // Normalized coordinates on (-1, 1)
    let x = x as f32 / (SIZE as f32 * 0.5) - 1.0;
    let y = -(y as f32) / (SIZE as f32 * 0.5) + 1.0;

    let camera_pos = Vec3 {
        z: -2.0,
        ..Vec3::zero()
    };
    let camera_dir = Vec3 {
        z: 1.0,
        ..Vec3::zero()
    };
    let up = Vec3 {
        y: 1.0,
        ..Vec3::zero()
    };
    let right = Vec3 {
        x: 1.0,
        ..Vec3::zero()
    };
    let rd = camera_dir
        + up * y
        + right * x;
    let rd = rd.normalize();
    let ro = camera_pos;

    let hit = trace(ro, rd);
    let f = hit.unwrap_or(0.0);

    let col = shade(color(ro + rd * f), up, normal(ro + rd * f));

    unsafe {
        buf_set(id * 4, rgb(col.x, col.y, col.z));
    }

    0
}

/// The playground refuses to compile it without a fake main function
/// This won't get called, though, because we're using the #[start] attribute
fn main() {}
;)
