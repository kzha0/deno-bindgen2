const { symbols } = Deno.dlopen(
    "/home/rico/Desktop/deno-bindgen2/target/release/libdeno_bindgen2_test.so",
    {
        _test_1: { parameters: [], result: "void" },
        _test_2: { parameters: ["buffer", "u32"], result: "pointer" },
        _test_2_ptr: { parameters: ["pointer"], result: "pointer" },
        _test_2_len: { parameters: ["pointer"], result: "u32" },
        _test_2_dealloc: { parameters: ["pointer"], result: "void" },
    },
);
export function test_1(): void {
    symbols._test_1();
}
export function test_2(arg_0: string): string {
    const arg_0_buf = new TextEncoder().encode(arg_0);
    const _test_2 = symbols._test_2(arg_0_buf, arg_0_buf.byteLength);
    const _test_2_ptr = symbols._test_2_ptr(_test_2!) as
        | Deno.PointerObject
        | null;
    const _test_2_len = symbols._test_2_len(_test_2!);
    const _test_2_buf = new Uint8Array(
        Deno.UnsafePointerView.getArrayBuffer(_test_2_ptr!, _test_2_len),
    );
    const _test_2_string = new TextDecoder("utf-8").decode(_test_2_buf);
    symbols._test_2_dealloc(_test_2!);
    return _test_2_string;
}
