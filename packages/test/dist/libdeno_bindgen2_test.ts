import type {
    RustF64,
    RustFnPtr,
    RustPtr,
    RustPtrMut,
    RustRef,
    RustRefMut,
    RustU8,
    RustUnsupportedType,
} from "./rust_type.ts";
import {
    RustBox,
    RustChar,
    RustPrototype,
    RustSlice,
    RustStr,
    RustString,
    RustVec,
} from "./rust_type.ts";

export const { symbols } = Deno.dlopen("target/debug/libdeno_bindgen2_test.so", {
    __test_1: { parameters: [], result: "void" },
    __test_2: { parameters: ["pointer"], result: "pointer" },
    __test_empty: { parameters: [], result: "void" },
    __test_unit: { parameters: [], result: "void" },
    __test_numeric: { parameters: ["f64"], result: "f64" },
    __test_bool: { parameters: ["bool"], result: "bool" },
    __test_char: { parameters: ["u32"], result: "u32" },
    __test_ptr: { parameters: ["pointer"], result: "pointer" },
    __test_ptr_mut: { parameters: ["pointer"], result: "pointer" },
    __test_ref: { parameters: ["pointer"], result: "pointer" },
    __test_mut: { parameters: ["pointer"], result: "pointer" },
    __test_box: { parameters: ["pointer"], result: "pointer" },
    __test_fn_ptr: { parameters: ["function"], result: "function" },
    __test_str: { parameters: ["pointer"], result: "pointer" },
    __test_string: { parameters: ["pointer"], result: "pointer" },
    __test_slice: { parameters: ["pointer"], result: "pointer" },
    __test_array: { parameters: ["pointer"], result: "pointer" },
    __test_vec: { parameters: ["pointer"], result: "pointer" },
    __test_path: { parameters: ["pointer"], result: "pointer" },
    __CustomType__test_self: { parameters: ["pointer"], result: "pointer" },
    __CustomType__test_ref_self: { parameters: ["pointer"], result: "pointer" },
    __CustomType__test_mut_self: { parameters: ["pointer"], result: "pointer" },
    __CustomType__test_other_self: {
        parameters: ["pointer", "pointer", "pointer"],
        result: "pointer",
    },
    __CustomType__test_other_type: { parameters: ["pointer"], result: "void" },
    __CustomType__drop: { parameters: ["pointer"], result: "void" },
    __RustString__new: { parameters: [], result: "pointer" },
    __RustString__from: { parameters: ["buffer", "u32"], result: "pointer" },
    __RustString__into_ptr: { parameters: ["pointer"], result: "buffer" },
    __RustString__into_len: { parameters: ["pointer"], result: "u32" },
    __RustString__push: { parameters: ["pointer", "buffer", "u32"], result: "void" },
    __RustString__drop: { parameters: ["pointer"], result: "void" },
});
export class SomeOtherType extends RustPrototype<SomeOtherType> {}
export class CustomType extends RustPrototype<CustomType> {
    test_self(): CustomType {
        const ptr = this.ptr;
        this.ptr = null;
        const out = symbols.__CustomType__test_self(ptr);
        return new CustomType(out!) as CustomType;
    }
    test_ref_self(): RustRef<CustomType> {
        const out = symbols.__CustomType__test_ref_self(this.ptr);
        return out! as RustRef<CustomType>;
    }
    test_mut_self(): RustRefMut<CustomType> {
        const ptr = this.ptr;
        this.ptr = null;
        const out = symbols.__CustomType__test_mut_self(ptr)!;
        this.ptr = ptr;
        return out! as RustRefMut<CustomType>;
    }
    test_other_self(arg_0: CustomType, arg_1: RustRefMut<CustomType>): RustRef<CustomType> {
        const out = symbols.__CustomType__test_other_self(this.ptr, arg_0.take(), arg_1);
        return out! as RustRef<CustomType>;
    }
    static test_other_type(arg_0: SomeOtherType) {
        symbols.__CustomType__test_other_type(arg_0.take());
    }
}
export function test_1() {
    symbols.__test_1();
}
export function test_2(arg_0: RustString): RustString {
    const out = symbols.__test_2(arg_0.take());
    return new RustString(out!) as RustString;
}
export function test_empty() {
    symbols.__test_empty();
}
export function test_unit() {
    symbols.__test_unit();
}
export function test_numeric(arg_0: RustF64): RustF64 {
    const out = symbols.__test_numeric(arg_0);
    return out! as RustF64;
}
export function test_bool(arg_0: boolean): boolean {
    const out = symbols.__test_bool(arg_0);
    return out! as boolean;
}
export function test_char(arg_0: RustChar): RustChar {
    const out = symbols.__test_char(arg_0.get());
    return new RustChar(out!) as RustChar;
}
export function test_ptr(arg_0: RustPtr<RustU8>): RustPtr<RustU8> {
    const out = symbols.__test_ptr(arg_0);
    return out! as RustPtr<RustU8>;
}
export function test_ptr_mut(arg_0: RustPtrMut<RustU8>): RustPtrMut<RustU8> {
    const out = symbols.__test_ptr_mut(arg_0);
    return out! as RustPtrMut<RustU8>;
}
export function test_ref(arg_0: RustRef<RustU8>): RustRef<RustU8> {
    const out = symbols.__test_ref(arg_0);
    return out! as RustRef<RustU8>;
}
export function test_mut(arg_0: RustRefMut<RustU8>): RustRefMut<RustU8> {
    const out = symbols.__test_mut(arg_0);
    return out! as RustRefMut<RustU8>;
}
export function test_box(arg_0: RustBox<RustU8>): RustBox<RustU8> {
    const out = symbols.__test_box(arg_0.take());
    return new RustBox<RustU8>(out!) as RustBox<RustU8>;
}
export function test_fn_ptr(
    arg_0: RustFnPtr<"extern fn (u8) -> u8">,
): RustFnPtr<"extern fn (u8) -> u8"> {
    const out = symbols.__test_fn_ptr(arg_0);
    return out! as RustFnPtr<"extern fn (u8) -> u8">;
}
export function test_str(arg_0: RustStr): RustStr {
    const out = symbols.__test_str(arg_0.take());
    return new RustStr(out!) as RustStr;
}
export function test_string(arg_0: RustString): RustString {
    const out = symbols.__test_string(arg_0.take());
    return new RustString(out!) as RustString;
}
export function test_slice(arg_0: RustSlice<RustU8>): RustSlice<RustU8> {
    const out = symbols.__test_slice(arg_0.take());
    return new RustSlice<RustU8>(out!) as RustSlice<RustU8>;
}
export function test_array(arg_0: RustUnsupportedType): RustUnsupportedType {
    const out = symbols.__test_array(arg_0);
    return out! as RustUnsupportedType;
}
export function test_vec(arg_0: RustVec<RustU8>): RustVec<RustU8> {
    const out = symbols.__test_vec(arg_0.take());
    return new RustVec<RustU8>(out!) as RustVec<RustU8>;
}
export function test_path(arg_0: RustUnsupportedType): RustUnsupportedType {
    const out = symbols.__test_path(arg_0);
    return out! as RustUnsupportedType;
}
