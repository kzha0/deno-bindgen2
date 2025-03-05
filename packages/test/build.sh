#!/usr/bin/bash
../core/build.sh $@
echo "finished build"

# -set out "-o ./src/mod.ts"

if [ "$1" = "--release" ]; then
    ../../target/release/deno-bindgen2 "$@"
else
    ../../target/debug/deno-bindgen2 "$@"
fi
# after build, run `deno run`