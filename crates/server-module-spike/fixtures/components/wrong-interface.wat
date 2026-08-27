(component
  (core module $module
    (func (export "handle") (param i32) (result i32)
      local.get 0))
  (core instance $instance (instantiate $module))
  (func (export "handle") (param "value" u32) (result u32)
    (canon lift (core func $instance "handle"))))
