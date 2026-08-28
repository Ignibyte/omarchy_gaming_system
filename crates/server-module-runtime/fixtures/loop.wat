(module
  (memory (export "memory") 1)
  (func (export "handle") (param i32 i64 i64) (result i32)
    (loop $again
      br $again)
    i32.const 0))
