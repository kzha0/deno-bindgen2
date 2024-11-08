
export { ValueMovedError } from "./util.ts";

const path = "target/x86_64-unknown-linux-gnu/release/libdeno_bindgen2_utils.so" as const;


// deno-lint-ignore-file
// let symbols: any;
// export function load(path: string = path) {
//     const { dlopen } = Deno;
//     const { symbols: _symbols } = dlopen(path, mod_symbols);
//     symbols = _symbols;
// }

export const { symbols } = Deno.dlopen(path, {
    __Metadata__rust_version: {
        parameters: [],
        result: "buffer",
    },
    __Metadata__rust_toolchain: {
        parameters: [],
        result: "buffer",
    },
    __Metadata__lib_name: {
        parameters: [],
        result: "buffer",
    },
    __Metadata__lib_version: {
        parameters: [],
        result: "buffer",
    },
});


export const RUST_VERSION = Deno.UnsafePointerView.getCString(
    symbols.__Metadata__rust_version() as Deno.PointerObject
);
export const RUST_TOOLCHAIN = Deno.UnsafePointerView.getCString(
    symbols.__Metadata__rust_toolchain() as Deno.PointerObject
);
export const LIB_NAME = Deno.UnsafePointerView.getCString(
    symbols.__Metadata__lib_name() as Deno.PointerObject
);
export const LIB_VERSION = Deno.UnsafePointerView.getCString(
    symbols.__Metadata__lib_version() as Deno.PointerObject
);

Deno.test("test metadata", () => {
    console.log("rust version   : ", RUST_VERSION);
    console.log("rust toolchain : ", RUST_TOOLCHAIN);
    console.log("lib name       : ", LIB_NAME);
    console.log("lib version    : ", LIB_VERSION);
});




