# exx

an extremely type-safe functional language that transpiles to Luau

## design

### functions

```rust
// functions are values, just like in Haskell or OCaml
let add: (number, number) -> number = (a, b) -> a + b;

// which means we can pass them around in higher-order functions
let apply: ((number) -> number, number) -> number =
    (f, x) -> f(x);

let square: (number) -> number = (x) -> x * x;
let result = apply(square, 5); // result = 25
```

### currying and partial application

```rust
// currying is supported
let multiply: (number) -> (number) -> number =
    (a) -> (b) -> a * b;

let double = multiply(2);
let result = double(5); // result = 10

// also supported with `fn` sugar
fn multiply(a: number) -> (number) -> number {
    return fn(b: number) -> number {
        return a * b;
    };
}

// partial application
fn addThreeNumbers(a: number, b: number, c: number) -> number {
    return a + b + c;
}

fn partialAdd(a: number) -> (number, number) -> number {
    return fn(b: number, c: number) -> number {
        return addThreeNumbers(a, b, c);
    };
}

let addFive = partialAdd(5);
let result = addFive(10, 20); // 35
```

### recursion

```rust
// recursion requires `rec` to allow self-referencing
let rec factorial: (number) -> number =
    (n) -> if n == 0 then 1 else n * factorial(n - 1);

// `rec` is not required when using `fn` sugar
fn factorial(n: number) -> number {
    if n == 0 {
        return 1;
    } else {
        return n * fctorial(n - 1);
    }
}
```

### type system

```rust
// strong static typing with type inference
let x: number = 42;
let message: string = "Hello, world!";

// function types are first-class
let fnType: (number) -> number = (x) -> x + 1;

// generic types
fn identity<T>(x: T) -> T {
    return x;
}

let numIdentity = identity(10);     // works for numbers
let strIdentity = identity("hi");   // works for strings

// union types
type Status = "loading" | "success" | "error";

let state: Status = "loading"; // type-safe
```

### algebraic data types (ADTs) and pattern matching

```rust
// ADTs allow sum types
type Option<T> = Some(T) | None;

// pattern matching ensures all cases are handled
fn getValue(opt: Option<number>) -> number {
    return match opt {
        Some(value) => value,
        None => 0
    };
}
```

### immutability

```rust
// immutable by default
let x = 10;
x = 20; // error: `x` is immutable

// use `var` for mutable values
var counter = 0;
counter = counter + 1; // OK
```

### modules

```rust
// modules help organize code
module Math {
    export fn square(x: number) -> number {
        return x * x;
    }
}

// importing a module
import Math;

let squared = Math.square(5); // 25
```

### interoperability with Luau

```rust
// can call Luau functions seamlessly
import luauModule;

let result = luauModule.someLuauFunction(10, "hello");

// can also define Luau-compatible types
type LuauObject = {
    name: string,
    age: number
};

let user: LuauObject = { name: "Alice", age: 25 };
```
