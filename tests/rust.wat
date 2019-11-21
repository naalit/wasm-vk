;; 4294836225 4294705156 4294574089 4294443024

;; This is a simple Rust program, converted to WASM and pasted in
;; I could set up a Cargo project, but it's so tiny I didn't bother
;; I've been just using the Playground to compile it
;; Source code is at the bottom

(module
  (type $t0 (func (result i32)))
  (import "spv" "id" (global $id i32)) ;; I changed it from a function to a global, because Rust doesn't support global imports
  (func $main (export "main") ;; I also removed the arguments and the result
    (local $l2 i32)
    (i32.store
      (local.tee $l2
        (get_global $id)) ;; Changed from a call to get_global
      (i32.mul
        (local.tee $l2
          (i32.sub
            (i32.const 65535)
            (i32.load
              (local.get $l2))))
        (local.get $l2)))) ;; Removed a return value of 0
  (start $main) ;; I added this too
  (table $T0 1 1 funcref)
  (memory $memory (export "memory") 16)
  (global $__data_end (export "__data_end") i32 (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576)))

;; Original source:
(;
#![feature(start)]
#![no_std]

#[link(wasm_import_module="spv")]
extern {
    fn trap() -> !;
    fn id() -> usize;
}

#[panic_handler]
fn handle_panic(_x: &core::panic::PanicInfo) -> ! {
    unsafe {
        trap()
    }
}

fn thread_id() -> &'static mut u32 {
    unsafe {
        core::mem::transmute(id())
    }
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    // We're going to reverse it, so we need the total.
    // For now it's hardcoded, but eventually we probably want to use SPIR-V builtins.
    const TOTAL: u32 = 65535; // 0..65536

    let slot = thread_id();

    let val = *slot;
    let reversed = TOTAL - val;
    let squared = reversed * reversed;

    *slot = squared;

    0
}

/// The playground refuses to compile it without a fake main function
/// This won't get called, though, because we're using the #[start] attribute
fn main() {}
;)
