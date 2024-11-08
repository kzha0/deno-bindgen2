import type { RustRef, RustRefMut, RustType } from "./core.ts";
import { RustPrototype } from "./core.ts";
import { ValueMovedError } from "./util.ts";

/**
 * Extensions to the base types
 */

const { symbols } = Deno.dlopen("target/x86_64-unknown-linux-gnu/release/libdeno_bindgen2_utils.so", {
    // <!-- deno-bindgen2-ffi-symbols-start -->
    __RustString__new: {
        parameters: [],
        result: "pointer",
    },
    __RustString__from: {
        parameters: ["buffer", "u32"],
        result: "pointer",
    },
    __RustString__into_ptr: {
        parameters: ["pointer"],
        result: "buffer",
    },
    __RustString__into_len: {
        parameters: ["pointer"],
        result: "u32",
    },
    __RustString__push: {
        parameters: ["pointer", "buffer", "u32"],
        result: "void",
    },
    __RustString__drop: {
        parameters: ["pointer"],
        result: "void",
    },
    // <!-- deno-bindgen2-ffi-symbols-end -->
});

// <!-- deno-bindgen2-alt-type-start -->

class RustBox<T extends RustType> extends RustPrototype<RustBox<T>> {}
class RustStr extends RustPrototype<RustStr> {}


/**
 * A class interface for interacting with Rust strings and converting them
 * into and from JavaScript strings
 */
class RustString extends RustPrototype<RustString> {

    constructor(ptr: Deno.PointerValue) {
        super(ptr);

    }

    protected borrow() {
        if (this.ptr) {
            const ptr = this.ptr as Deno.PointerObject;
            return ptr as Deno.PointerObject<RustRef<RustString>>;
        } else {
            throw new ValueMovedError();
        }
    }

    protected borrow_mut(callback: (ref: RustRefMut<RustString>) => void) {
        if (this.ptr) {
            const ptr = this.ptr as Deno.PointerObject;
            this.ptr = null;
            callback(ptr as RustRefMut<RustString>);
            this.ptr = ptr as Deno.PointerObject<RustPrototype<RustString>>;
        } else {
            throw new ValueMovedError();
        }
    }

    /**
     * Create and allocate an empty RustString instance
     */
    static new() {
        const ptr = symbols.__RustString__new();
        return new RustString(ptr!);
    }

    /**
     * Create a RustString from a JavaScript string
     */
    static from(string: string) {
        const buf = new TextEncoder().encode(string);
        const ptr = symbols.__RustString__from(buf, buf.byteLength);
        return new RustString(ptr!);
    }

    /**
     * Get this RustString's contents as a JavaScript string
     */
    into() {
        const ptr = symbols.__RustString__into_ptr(this.borrow());
        const len = symbols.__RustString__into_len(this.borrow());
        const buf = Deno.UnsafePointerView.getArrayBuffer(ptr!, len);
        const string = new TextDecoder().decode(buf);
        return string;
    }

    /**
     * Append a JavaScript string to the contents of this RustString
     */
    push(string: string) {
        this.borrow_mut((ref_mut) => {
            const buf = new TextEncoder().encode(string);
            symbols.__RustString__push(ref_mut, buf, buf.byteLength);
        });
    }

    [Symbol.dispose]() {
        const ptr = this.take();
        this.ptr = null;
        symbols.__RustString__drop(ptr);
    }
}

// <!-- deno-bindgen2-ignore-start -->
/*
[!ISSUE] provide implementation to interface with collection types and data
structures that are generic
there is no representation for generic types in the C ABI, so generic types
must be monomorphized into their concrete types.one way to make a similar
representation in TS is by monomorphizing the type as well.

Another could be through extern types wherein types are represented behind
an opaque pointer. their size and alignment is taken to properly instantiate
the Vec<T> type

See issue for more info on extern types
https://github.com/rust-lang/rust/issues/43467
*/
// <!-- deno-bindgen2-ignore-end -->
class RustSlice<T extends RustType> extends RustPrototype<RustSlice<T>> {}
class RustVec<T extends RustType> extends RustPrototype<RustVec<T>> {}
class RustTuple<T extends RustType[]> extends RustPrototype<RustTuple<T>> {}

// <!-- deno-bindgen2-alt-type-end -->

export {
    RustBox,
    RustStr,
    RustString,
    RustSlice,
    RustVec,
    RustTuple,
}

Deno.test("test alloc", () => {
    using string = RustString.from("Hello from Deno to Rust!");
    console.log(string.into());
});

Deno.test("test null", () => {
    const string = RustString.from("empty");
    string.take();
    string.take();
});

Deno.test("test dispose", () => {
    using string = RustString.from("empty");
    // why is the error not displaying properly?
    string.take();
    string.take();
});

Deno.test("test mut", () => {
    using string = RustString.new();
    string.push("Hello");
    console.log(string.into());
    string.push(", world!");
    console.log(string.into());
});
