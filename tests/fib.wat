;; 1 1 2 3 5 8 13 21
(; An iterative Fibonacci generator ;)
(; The algorithm is roughly
  fn fib(i) {
    let mut a = 0;
    let mut b = 1;

    for 0..i {
      (a, b) = (b, a + b);
    }

    b
  }
;)
(module
  (start $main)
  (import "spv" "id" (global $id i32))
  (memory 1)

  (func $main
    (local $ptr i32)
    (local $a i32)
    (local $b i32)
    (local $i i32)
    (; For swapping a and b ;)
    (local $tmp i32)

    (local.set $ptr
      (i32.mul
        (i32.const 4)
        (global.get $id)))

    (local.set $i
      (i32.const 0))
    (local.set $a
      (i32.const 0))
    (local.set $b
      (i32.const 1))

    (loop
      (if (i32.eq (local.get $i) (global.get $id))
      (then
        (; We've done the right number of iterations, so break ;)
        (i32.store (local.get $ptr) (local.get $b))
      ) (else
        (; Get the next number and continue ;)
        (local.set $tmp (local.get $a))
        (local.set $a (local.get $b)) (; a = b ;)
        (local.set $b (i32.add (local.get $tmp) (local.get $b))) (; b = old_a + b ;)
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (; continue ;)
        (br 1)
      ))
    )
  )
)
