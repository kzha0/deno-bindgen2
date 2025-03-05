import type { RustU64 } from "./rust_type.ts";
import { RustPrototype } from "./rust_type.ts";

export const { symbols } = Deno.dlopen("target/release/libbench.so", {
    __add: { parameters: ["u64", "u64"], result: "u64" },
    __RustString__new: { parameters: [], result: "pointer" },
    __RustString__from: { parameters: ["buffer", "u32"], result: "pointer" },
    __RustString__into_ptr: { parameters: ["pointer"], result: "buffer" },
    __RustString__into_len: { parameters: ["pointer"], result: "u32" },
    __RustString__push: { parameters: ["pointer", "buffer", "u32"], result: "void" },
    __RustString__drop: { parameters: ["pointer"], result: "void" },
});

export function add(arg_0: RustU64, arg_1: RustU64): RustU64 {
    const out = symbols.__add(arg_0, arg_1);
    return out! as RustU64;
}
