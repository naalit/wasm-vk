;; 12 11 10 3 9 5
;; Tests using linear memory, both for data section loads and dynamic loads and stores
;; Rust source once again at the bottom
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
    (local $l2 i32)
    (local.set $l2
      (i32.load
        (i32.add
          (local.get $p1)
          (i32.const -4))))
    (block $B0
      (br_if $B0
        (i32.gt_u
          (local.tee $p1
            (i32.load
              (local.get $p1)))
          (i32.const 4)))
      (br_if $B0
        (i32.eqz
          (i32.and
            (i32.shr_u
              (i32.const 23)
              (i32.and
                (local.get $p1)
                (i32.const 255)))
            (i32.const 1))))
      (local.set $p1
        (i32.load
          (i32.add
            (i32.shl
              (local.get $p1)
              (i32.const 2))
            (i32.const 1048576)))))
    (i32.store offset=4
      (local.get $p0)
      (local.get $p1))
    (i32.store
      (local.get $p0)
      (local.get $l2)))
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
  (memory $memory (export "memory") 17)
  (global $g0 (mut i32) (i32.const 1048576))
  (global $__data_end (export "__data_end") i32 (i32.const 1048596))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048596))
  (data $d0 (i32.const 1048576) "\0c\00\00\00\0b\00\00\00\0a\00\00\00\0c\00\00\00\09\00\00\00"))
;; Rust source:
(;
#![feature(start)]
#![no_std]

#[link(wasm_import_module="spv")]
extern {
    fn trap() -> !;
    fn id() -> usize;

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

struct Pair(u32, u32);

#[inline(never)]
fn return_struct(i: &u32) -> Pair {
    Pair(unsafe { *((i as *const u32 as usize - 4) as *const u32) }, match *i {
        0 => 12,
        1 => 11,
        2 => 10,
        4 => 9,
        x => x,
    })
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    let val = unsafe { buf_get(id() * 4) };
    let pair = return_struct(&val);
    let new_val = pair.0 + pair.1;

    unsafe {
        buf_set(id() * 4, new_val);
    }

    0
}

/// The playground refuses to compile it without a fake main function
/// This won't get called, though, because we're using the #[start] attribute
fn main() {}
;)
