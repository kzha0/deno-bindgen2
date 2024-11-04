# Assumptions and Limitations

This section describes some assumptions and limitations when using the `deno_bindgen2` tool

> [!WARNING] Disclaimer
> Any discussion about Rust and its internal workings are based off the author's best understanding of the topic. Nothing written here should be construed as complete technical descriptions or explanations of the Rust language and its implementation

## Reserved Types

> [!NOTE] TLDR
> When using Rust's built-in types...
>
> ❌ Don't use their fully qualified path `std::primitive::u8`\
> ❌ Don't import `std::collections` nor write `collections::Vec<T>`\
> ❌ Don't alias these types, like `type String = &[u8]`
>
> ✅ Instead, just write `u8`, `Vec<T>`, or `String` to make sure the tool works as intended\
> Consult with the full [list of reserved types](#reserved-type-list) for more info

When using the `deno_bindgen` macro, users must take into account the following *reserved* types and make sure that the type expression matches the type it expects, without collisions resulting from type aliasing

<details open>
<summary>Full list of reserved type expressions</summary>
<a name="reserved-type-list"></a>

| Expected fully qualified path | Reserved type name/shorthand |
|-|-|
| `std::primitive::u8`    | `u8`     |
| `std::primitive::u16`   | `u16`    |
| `std::primitive::u32`   | `u32`    |
| `std::primitive::u64`   | `u64`    |
| `std::primitive::usize` | `usize`  |
| `std::primitive::i8`    | `i8`     |
| `std::primitive::i16`   | `i16`    |
| `std::primitive::i32`   | `i32`    |
| `std::primitive::i64`   | `i64`    |
| `std::primitive::isize` | `isize`  |
| `std::primitive::f32`   | `f32`    |
| `std::primitive::f64`   | `f64`    |
| `std::primitive::bool`  | `bool`   |
| `std::primitive::char`  | `char`   |
| `std::primitive::str`   | `str`    |
| `std::string::String`   | `String` |
| `std::boxed::Box`       | `Box<T>` |
| `std::collections::Vec` | `Vec<T>` |

</details>

### Detailed explanation

`deno_bindgen2` performs some checking on the user's code (specifically on function signatures[^1]) and detects if any *type expression* matches against its internal list of *reserved* types. For example, say you wrote:

```rust
#[deno_bindgen]
fn my_ffi_function(number: u8) {
    println!("{number}");
}

```

The macro will process this input code and see the `u8` type. During macro expansion, this type will be written as its fully qualified path like below:

```rust
#[deno_bindgen]
fn my_ffi_function(number: u8) {
    println!("{number}");
}

// this is what `deno_bindgen2` generated
#[unsafe(no_mangle)]
extern "C" __my_ffi_function(arg_0: ::std::primitive::u8) {
                                   // ^ notice the expansion here
    my_ffi_function(arg_0);
 // ^ the generated function is just a wraper around the original function
}
```

### Rationale

To understand the rationale behind this imposed limitation, let us look at a case where this expectation is broken. Consider the following code as an example :

```rust
#[deno_bindgen]
fn my_ffi_function(string: String) {}

mod my_mod {
    type String = &[u8];
      // ^ an alias for `String` is created!

    #[deno_bindgen]
    fn my_ffi_function(shadowed_string: String)
}
```

In this example, the type `String` was given two meanings

- `std::string::String` which was brought into scope by the [prelude section](https://doc.rust-lang.org/reference/names/preludes.html)
- `&[u8]` a type alias defined by the user

During compilation, the Rust compiler does a process called [name resolution](https://rustc-dev-guide.rust-lang.org/name-resolution.html). It looks at the source code as a whole to know exactly what the type `String` refers to. Procedural macros like `deno_bindgen2`, however, do not have this visibility as it only process whatever section of source code it was fed as input:

```rust
// `deno_bindgen2` assumes that the reserved types come from the prelude
extern crate std;
use std::prelude::*;

// -----------------------------------------+-- macro scope
//                                          |
#[deno_bindgen] //                          |   this is what deno_bindgen is fed
fn my_ffi_function(string: String) {} //    |   and doesn't know anything about
//                                          |   whatever is outside this box :)
// -----------------------------------------+

mod my_mod {
    type String = [u8];
    // ...
}
```

Thus, this tool somewhat takes away a (hopefully) negligible amount of freedom when writing code, especially when it comes to making type aliases for built-in types from the standard library. It makes some assumptions about what the user's intentions are when writing those reserved types, and it enforces an opinionated way of writing code when using this tool.

> [!NOTE]
> In the future, `deno_bindgen2` may provide partial support for aliased types through *globally-scoped* macros or those that can encapsulate a significant part of the source code. Through an implementation like:
>
> ```rust
> deno_bindgen2! {
>     use my_mod::String;
>
>     pub fn my_ffi_function(string: String) {}
> }
> ```
>
> ...we can give the macro access to a wider scope and possibly detect any `use` statement to override the list of reserved types. However, this will require significant work as it implies mimicking the Rust compiler's name resolution infrastructure.

## Type Paths

As of current, `deno_bindgen2` supports most type paths, save for a few reserved types mentioned in the previous section. This is because the tool doesn't have a name resolver

```rust
use my_mod::CustomType;

#[deno_bindgen]
impl CustomType {
    fn my_ffi_fn(input: my_mod::CustomType) {}
                     // ^ without name resolution, paths won't work as intended
}

```

Another limitation is that implement blocks must refer to the implementing type by their bare identifier. Meaning, users should not use type paths on implement blocks

```rust
use my_mod::CustomType;

#[deno_bindgen]
impl CustomType {
  // ^ this is the proper way
    fn my_ffi_fn(self, other_self: my_mod::CustomType) {}
}

#[unsafe(no_mangle)]
extern "C" fn __CustomType__my_ffi_fn(arg_0: CustomType, arg_1: my_mod::CustomType) {
    CustomType::my_ffi_fn(arg_0, arg_1);
                       // ^ the `self` parameter
}
```

Without a name resolution implementation, `deno_bindgen2` cannot properly translate each user-defined type without guarantee that no user-defined type is defined more than once in the same scope once implemented in TypeScript

## Polymorphic Code

For implement blocks and `UserDefined` types, `deno_bindgen2` only supports types that are bare identifiers (those without any type arguments). This is because the tool doesn't have a way to uniquely identify types like `UserDefined<T>` where a symbol has to be generated for every variant of `T`

Illustration

```rust
// without type arguments
#[deno_bindgen]
impl CustomType {
    fn my_ffi_function () {}
}

extern "C" __CustomType__my_ffi_function() {}
```

```rust
// with type arguments
#[deno_bindgen]
impl CustomType<Vec<[u8]>> {
    fn my_ffi_function () {}
}

extern "C" __CustomType__???__my_ffi_function() {}
                      // ^ no way to sanely name this part yet!
```

[^1]: A function signature `fn my_fn<T>(arg: usize) -> T` describes the name `my_fn`, arguments `(arg: usize)`, result `-> T`, and, in the case of generic functions, the generic parameters `T`, of a function
