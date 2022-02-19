(module
    (memory $mem 1 10)

    (func $_start (result i32)
        (call $fibonacci (i32.const 1))
    )

    (func $dummy_func (result i32)
        (local $dummy i32)

        (local.set $dummy (i32.const 100))

        (i32.add (local.get $dummy) (i32.const 100))
    )

    (func $fibonacci (param $number i32) (result i32)
        ;; base condition 1
        (if (i32.eq (local.get $number) (i32.const 0))
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

    ;; Dummy data
    (data $dummy_data (i32.const 4) "\00\01\02\03")

    (export "_start" (func $_start))
)
