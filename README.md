# expressi

Expression-oriented toy programming language written in Rust

```
let add = a -> b -> a + b;
let succ = add(1);
let v = succ(succ(succ(1)));

let f = if v == 4 {
  a -> succ(a)
} else {
  a -> a
};
let x = f(10)
```

In this example, `x` is evaluated to `11`.

## TODO

- Refine EIR
  - Delete `Typed` constructor and provide another way to express typed value
  - Convert `Follow`, `Bind`, `Scope` to let-in expression
  - Implement `Printer` as an implementation of `fmt::Display`
- Refine errors
  - Property organize error variants
  - Get rid of `unwrap` completely
  - Point where the cause is
- Add `EvalConstant` transformer which calculates compile-time value
- Implement operators as functions
- User-defined types
  - Tuple
  - Enum
  - Struct
- Multi-line input in REPL

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
