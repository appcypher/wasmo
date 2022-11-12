(module
  (func $add (param $lhs i32) (param $rhs i32) (result i32)
    (i32.add (local.get 0) (local.get 1))
  )

  (func $sub (param $lhs i32) (param $rhs i32) (result i32)
    (i32.sub (local.get 0) (local.get 1))
    return
    nop
    nop
  )

  (func $_start (result i32)
    (call $add (i32.const 45) (i32.const 5))
  )

  (export "add" (func $add))
  (export "sub" (func $sub))
  (export "_start" (func $_start))
)
