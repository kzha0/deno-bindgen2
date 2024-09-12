# Types

## C ABIs and Fundamentals Type/Trivial Type Restrictions

The C Application Binary Interface (ABI) restricts parameters to only fundamental or trivial types. Fundamental types are the basic integral and floating-point types, and trivial types are structures whose fields are all fundamental types.

The reasons for this restriction lie in the fundamental characteristics of C. C was initially devised as a system programming language that would run on a wide range of hardware platforms. Keeping the language simple and compact was vital to ensure its portability across different machines.

### Relevance of the `Copy` trait in Rust

The `Copy` trait in Rust allows for a value's copy to be cheaply generated without doing a deep copy. A type can be marked as implementing `Copy` if its values can be read or written without a reference.

The relevance of the `Copy` trait comes into play when dealing with types that are trivially copyable. In Rust, a type is trivially copyable if it is a fundamental type (like integers, floats, and booleans) or a tuple that only contains trivially copyable types. Trivially copyable types can be copied using a simple bitwise copy, which is more efficient and can be done without invoking any constructors or destructors, which is crucial for performance in Rust.

### Relevance of `TriviallyCopyable` in CPP

CPP's `TriviallyCopyable` named requirement is similar to Rust's `Copy` trait. A type is considered trivially copyable if it can be copied with `std::memcpy`. This is a simple bitwise copy and doesn't involve any special member functions like constructors and destructors, leading to performance benefits.

### Limitations of `memcpy` for complex types

The `memcpy` function is restricted in copying complex types that go beyond a single type. `memcpy` and similar functions work by making a bitwise copy of the data, but if the data is complex and crosses multiple types or contains non-trivial data, `memcpy` will not work properly. For instance, if a structure contains pointers or handles to other data, attempting to copy it with `memcpy` could result in invalid references or memory leaks.

### The Need for Three Types in FFI Generation

Given these restrictions, the need for defining three types comes down to how the Rust code is transformed into Deno code through the Foreign Function Interface (FFI) generator. These types are:

1. `Native`: This type refers to Rust's trivially copyable types and fundamental types. They can be copied and moved directly, as they can be copied with a simple bitwise copy and do not contain complex data that would need special handling.

2. `Extended`: This refers to complex Rust data types that contain multiple fundamental or trivially copyable types. These types have to be handled more carefully and may require custom copying mechanisms or further decomposition before they can be safely used with FFI.

3. `Unsupported`: These are types that cannot be handled by the FFI generator, most likely due to them containing unsupported data structures or having methods that are not trivially copyable. These types can't be used with the FFI.

These classifications help manage and control the complexity that can stem from converting Rust code into Deno code, while also ensuring that all types can be handled without causing errors or memory leaks.

*Disclaimer: The information presented here is based on a general understanding of the concepts and may require further exploration or specific context to fully comprehend.*

### Further reading

[Copy trait](https://doc.rust-lang.org/std/marker/trait.Copy.html)

[Trivial return values](https://itanium-cxx-abi.github.io/cxx-abi/abi.html#non-trivial-return-values)
