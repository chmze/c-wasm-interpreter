# c-wasm-interpreter

Web-based interpret of a subset of the C programming language subset written with React (TypeScript) and WASM in Rust.

## Features

- [x] Primitive data types
- [x] Type conversion
- [ ] Numerals (remaining: hex, octal)
- [x] Custom memory model
- [x] Conditionals
- [ ] Loops (remaining: for)
- [ ] Expressions (various missing)
- [x] Call stack, functions, and recursion
- [ ] Printing
- [x] Comprehensive unit tests
- [ ] Website (remaining: UI)

### Planned for next versions

- [ ] Arrays
- [ ] Structs, unions
- [ ] Pointers
- [ ] Standard library functions

## Execution demos

Calculates **Fibonacci numbers recursively**:

```c
int fib(int n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
```

For `fib(10)`, the produced memory is:

`[55, 0, 0, 0, 8, 0, 0, 0, 6, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]`

The first 4 bytes contain the result (`fib(10) = 55`); remaining memory demonstrates surviving values from other stack frames.

## Instructions

### Install

`npm i --legacy-peer-deps`

### Run

`npm run dev`

### Test

`cargo test`
