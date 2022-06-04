(module
  ;; def control(a, b):
  ;;     if a > b:
  ;;         return 0
  ;;     tmp = 0
  ;;     while True:
  ;;         a += 1
  ;;         b -= 1
  ;;         if a >= b:
  ;;             tmp = a
  ;;             break
  ;;         else:
  ;;             continue
  ;;     return tmp
  (func $control (param i64 i64) (result i64 i64 i64 i64)
    ;; === Locals ===
    (local $result i64)
    (local $count i64)

    ;; === Body ===
    ;; (if (i64.gt_s (local.get 0) (local.get 1))
    ;;   (then (return (local.get $result) (i64.const 0) (local.get 0) (local.get 1)))
    ;; )

    ;; (block
    ;;   (loop
    ;;     (local.set 0 (i64.add (local.get 0) (i64.const 1)))
    ;;     (local.set 1 (i64.sub (local.get 1) (i64.const 1)))
    ;;     (local.set $count (i64.add (local.get $count) (i64.const 1)))
    ;;     (if (i64.ge_s (local.get 0) (local.get 1))
    ;;       (then
    ;;         (local.set $result (local.get 0))
    ;;         (br 2)
    ;;       )
    ;;       (else (br 1))
    ;;     )
    ;;   )
    ;; )

    ;; (return (local.get $result) (local.get $count) (local.get 0) (local.get 1))
  )

  (export "control" (func $control))
)
