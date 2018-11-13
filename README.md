# expressi

Expression-oriented toy programming language written in Rust

```
let mut a = 2;
let mut b = a + 10;
let c = if a == b {
  a = a + 10;
  20
} else {
  b = b + 20;
  10
};
let x = c + b
```

In this example, `x` is evaluated to `42`.

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
