// <!-- deno-bindgen2-content-start -->

class ValueMovedError extends Error {
    constructor() {
        super("attempted to access a pointer value after it was moved. https://doc.rust-lang.org/error_codes/E0382.html");
    }
}

// <!-- deno-bindgen2-content-end -->

export { ValueMovedError };

Deno.test("ValueMovedError", () => {
    throw new ValueMovedError();
})
