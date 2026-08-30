(module
  (memory (export "memory") 1)
  (global i32 (i32.const 2))
  (func (export "handle") (param i32 i64 i64) (result i32)
    i32.const 0
    i32.const 1
    i32.store
    i32.const 8
    local.get 1
    i64.store
    i32.const 16
    i64.const 7
    i64.store
    i32.const 0))
