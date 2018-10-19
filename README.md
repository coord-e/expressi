# expressi

Expression-oriented toy programming language written in Rust

```
a = 2;
b = a + 10;
c = if a == b {
  a = a + 10;
  20
} else {
  b = b + 20;
  10
};
x = c + b
```

In this example, `x` is evaluated to `42`.

