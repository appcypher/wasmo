(module
    (func $_start (result i32)
        (call $fibonacci (i32.const 10))
    )

    (func $fibonacci (param $number i32) (result i32)
        ;; base condition 1
        (if (i32.eqz (local.get $number))
            (then (return (i32.const 1)))
        )

        ;; base condition 2
        (if (i32.eq (local.get $number) (i32.const 1))
            (then (return (i32.const 1)))
        )

        ;; recursive call
        (i32.add
            (call $fibonacci
                (i32.sub (local.get $number) (i32.const 1))
            )

            (call $fibonacci
                (i32.sub (local.get $number) (i32.const 2))
            )
        )
    )

    (export "_start" (func $_start))
)
