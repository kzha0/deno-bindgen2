import { ValueMovedError } from "./util.ts";

/**
 * Collection of types for representing Rust primitives and base classes/types
 * for extending these types
 */

// <!-- deno-bindgen2-content-start -->

type RustU8 = number;
type RustU16 = number;
type RustU32 = number;
type RustI8 = number;
type RustI16 = number;
type RustI32 = number;
type RustF32 = number;
type RustF64 = number;
type RustU64 = bigint;
type RustUsize = bigint;
type RustI64 = bigint;
type RustIsize = bigint;

class RustChar {
    protected char: number | null = null;

    constructor(char: number) {
        this.char = char;
    }

    get() {
        return this.char!;
    }

    /**
     * Create a RustChar instance from the first character of a JavaScript
     * string. Discards any succeeding character.
     */
    static from(char: string) {
        return new RustChar(char.charCodeAt(0));
    }

    /**
     * Get the string representation of this Rust char
     */
    into() {
        return String.fromCharCode(this.char!);
    }
}

type RustPrimitive = number | bigint | boolean | void | RustChar;

/**
 * A base class for representing complex types such as String, str, Vec,and
 * user-defined types
 */
abstract class RustPrototype<T = unknown> {
    protected ptr: Deno.PointerValue<RustPrototype<T>> = null;

    constructor(ptr: Deno.PointerValue) {
        this.ptr = ptr! as Deno.PointerObject<RustPrototype<T>>;
    }

    /**
     * Consumes the pointer and empties the contents of this object, making it
     * unusable. Any succeeding calls to this method will throw an error.
     */
    take() {
        if (this.ptr) {
            const ptr = this.ptr;
            this.ptr = null;
            Object.freeze(this);
            return ptr;
        } else {
            throw new ValueMovedError();
        }
    }
}

type RustFnPtr<T extends string | null = null> = Deno.PointerObject<RustFnPtr<T>>;
type RustPtr<T extends RustType | unknown = unknown> = Deno.PointerObject<RustPtr<T>>;
type RustPtrMut<T extends RustType | unknown = unknown> = Deno.PointerObject<RustPtrMut<T>>;
type RustRef<T extends RustType | unknown = unknown> = Deno.PointerObject<RustRef<T>>;
type RustRefMut<T extends RustType | unknown = unknown> = Deno.PointerObject<RustRefMut<T>>;

type RustReferenceType = RustFnPtr | RustPtr | RustRef | RustPtrMut | RustRefMut ;

type RustUnsupportedType = Deno.PointerObject<RustUnsupportedType>;

/**
 * A base representation of possible Rust types. Use this type when extending
 * upon opaque pointer objects with generic parameters
 * (i.e. `RustBox<T extends RustType>`)
 */
type RustType =
    | RustPrimitive // numeric types
    | RustPrototype // containers, smart pointers, user-defined types
    | RustReferenceType // reference types
    | RustUnsupportedType; // fallback unknown type

// <!-- deno-bindgen2-content-end -->

// <!-- deno-bindgen2-alt-type-start -->

class RustBox<T extends RustType> extends RustPrototype<RustBox<T>> {}
class RustStr extends RustPrototype<RustStr> {}
class RustString extends RustPrototype<RustString> {}
class RustSlice<T extends RustType> extends RustPrototype<RustSlice<T>> {}
class RustVec<T extends RustType> extends RustPrototype<RustVec<T>> {}
class RustTuple<T extends RustType[]> extends RustPrototype<RustTuple<T>> {}

// <!-- deno-bindgen2-alt-type-end -->

export type {
    RustU8,
    RustU16,
    RustU32,
    RustI8,
    RustI16,
    RustI32,
    RustF32,
    RustF64,
    RustU64,
    RustUsize,
    RustI64,
    RustIsize,
    RustFnPtr,
    RustPtr,
    RustPtrMut,
    RustRef,
    RustRefMut,
    RustUnsupportedType,
    RustType,
};

export {
    RustChar,
    RustPrototype,
    RustBox,
    RustStr,
    RustString,
    RustSlice,
    RustVec,
    RustTuple,
}
