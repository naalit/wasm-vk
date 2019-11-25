;; This one calculates the Mandelbrot fractal
;; It was generated with the Rust playground, and then I ran the 'fix_wat.py' script on it to make it use wasm-vk's API
;; The Rust source is at the bottom
(module
  (start $main)
  (type $t0 (func (result i32)))
  (type $t1 (func (param f32) (result f32)))
  (type $t2 (func (param i32 i32) (result i32)))
  (import "spv" "id" (global $id i32))

  (func $main (export "main")
    (local $l2 i32) (local $l3 f32) (local $l4 f32) (local $l5 f32) (local $l6 i32) (local $l7 f32) (local $l8 f32) (local $l9 f32) (local $l10 f32)
    (local.set $l2
      (i32.shl
        (get_global $id)
        (i32.const 2)))
    (local.set $l3
      (f32.add
        (f32.mul
          (f32.convert_i32_u
            (i32.and
              (get_global $id)
              (i32.const 255)))
          (f32.const 0x1p-7 (;=0.0078125;)))
        (f32.const -0x1p+0 (;=-1;))))
    (local.set $l4
      (f32.add
        (f32.mul
          (f32.convert_i32_u
            (i32.shr_u
              (get_global $id)
              (i32.const 8)))
          (f32.const 0x1p-7 (;=0.0078125;)))
        (f32.const -0x1p+0 (;=-1;))))
    (local.set $l5
      (f32.const 0x0p+0 (;=0;)))
    (local.set $l6
      (i32.const 101))
    (local.set $l7
      (f32.const 0x0p+0 (;=0;)))
    (local.set $l8
      (f32.const 0x0p+0 (;=0;)))
    (block $B0
      (loop $L1
        (br_if $B0
          (i32.eqz
            (local.tee $l6
              (i32.add
                (local.get $l6)
                (i32.const -1)))))
        (block $B2
          (br_if $B2
            (f32.eq
              (local.tee $l10
                (f32.sqrt
                  (local.tee $l9
                    (f32.add
                      (f32.mul
                        (local.tee $l8
                          (f32.add
                            (local.get $l3)
                            (f32.sub
                              (f32.mul
                                (local.get $l8)
                                (local.get $l8))
                              (f32.mul
                                (local.get $l7)
                                (local.get $l7)))))
                        (local.get $l8))
                      (f32.mul
                        (local.tee $l7
                          (f32.add
                            (local.get $l4)
                            (f32.add
                              (local.tee $l7
                                (f32.mul
                                  (local.get $l7)
                                  (local.get $l8)))
                              (local.get $l7))))
                        (local.get $l7))))))
              (local.get $l10)))
          (local.set $l10
            (f32.sqrt
              (local.get $l9))))
        (local.set $l5
          (f32.add
            (local.get $l5)
            (f32.const 0x1.47ae14p-7 (;=0.01;))))
        (br_if $L1
          (i32.xor
            (f32.gt
              (local.get $l10)
              (f32.const 0x1p+2 (;=4;)))
            (i32.const 1)))))
    (block $B3
      (block $B4
        (br_if $B4
          (i32.eqz
            (i32.and
              (f32.lt
                (local.tee $l7
                  (f32.mul
                    (local.get $l5)
                    (f32.const 0x1.fep+7 (;=255;))))
                (f32.const 0x1p+32 (;=4.29497e+09;)))
              (f32.ge
                (local.get $l7)
                (f32.const 0x0p+0 (;=0;))))))
        (local.set $l6
          (i32.trunc_f32_u
            (local.get $l7)))
        (br $B3))
      (local.set $l6
        (i32.const 0)))
    (i32.store
      (local.get $l2)
      (i32.or
        (i32.or
          (i32.or
            (i32.shl
              (local.get $l6)
              (i32.const 8))
            (local.get $l6))
          (i32.shl
            (local.get $l6)
            (i32.const 16)))
        (i32.const -16777216)))
    )
  (table $T0 1 1 funcref)
  (memory $memory (export "memory") 16)
  (global $__data_end (export "__data_end") i32 (i32.const 1048576))
  (global $__heap_base (export "__heap_base") i32 (i32.const 1048576)))

;; Rust source:
(;
#![feature(start)]
#![no_std]

#[link(wasm_import_module="spv")]
extern {
    fn trap() -> !;
    fn id() -> usize;
    fn sqrt(_: f32) -> f32;
}

#[panic_handler]
fn handle_panic(_x: &core::panic::PanicInfo) -> ! {
    unsafe {
        trap()
    }
}

fn thread_id() -> &'static mut u32 {
    unsafe {
        core::mem::transmute(id() * 4)
    }
}

fn length(x: f32, y: f32) -> f32 {
    unsafe { sqrt(x*x + y*y) }
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    let slot = thread_id();

    let x = unsafe { id() } % 256;
    let y = unsafe { id() } / 256;

    // Normalized coordinates on (-1, 1)
    let x = x as f32 / 128.0 - 1.0;
    let y = y as f32 / 128.0 - 1.0;

    let mut zx: f32 = 0.0;
    let mut zy: f32 = 0.0;

    let mut col: f32 = 0.0;

    for _ in 0..100 {
        col += 1.0 / 100.0;

        zx = zx * zx - zy * zy + x;
        zy = zy * zx + zx * zy + y;

        if length(zx, zy) > 4.0 {
            break
        }
    }

    let c = (col * 255.0) as u32;
    let new_val = 255 << 24 // Alpha = 1.0
                | c << 16
                | c << 8
                | c;

    *slot = new_val;

    0
}

/// The playground refuses to compile it without a fake main function
/// This won't get called, though, because we're using the #[start] attribute
fn main() {}

;)
