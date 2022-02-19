(module
    (func $_start (result i32)
        (call $add (i32.const 45) (i32.const 5))
    )

    (func $add (param i32 i32) (result i32)
       (i32.add (local.get 0) (local.get 1))
    )

    (export "add" (func $add))
    (export "_start" (func $_start))
)
