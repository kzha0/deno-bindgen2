// deno-lint-ignore-file
import { symbols } from "./libbench.ts";

class ValueMovedError extends Error {
    constructor() {
        super("attempted to access a pointer value after it was moved. https://doc.rust-lang.org/error_codes/E0382.html");
    }
}


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


class RustSlice<T extends RustType> extends RustPrototype<RustSlice<T>> {}
class RustVec<T extends RustType> extends RustPrototype<RustVec<T>> {}
class RustTuple<T extends RustType[]> extends RustPrototype<RustTuple<T>> {}

export type { RustU64 };
export { RustPrototype };
