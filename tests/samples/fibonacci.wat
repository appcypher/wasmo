(module
    (memory $mem 1 10)

    (table $table 1 10 funcref)

    (func $_start (result i32)
        (call $fibonacci (i32.const 10))
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

    ;; init.
    (data $data (memory $mem) (offset (i32.const 0)) "\00\01\02\03")

    (elem $elem (table $table) (offset (i32.const 0)) funcref (item (i32.const 0)))

    (export "_start" (func $_start))
)
