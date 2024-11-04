#!/usr/bin/fish
../core/build.fish $argv
echo "finished build"

# -set out "-o ./src/mod.ts"

if test "$argv[1]" = "--release"
    ../../target/release/deno-bindgen2 $argv # $out
else
    ../../target/debug/deno-bindgen2 $argv # $out
end
# after build, run `deno run`