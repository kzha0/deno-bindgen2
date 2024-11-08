#!/usr/bin/fish

RUSTFLAGS="-Zdylib-lto -Zlocation-detail=none -Zfmt-debug=none -C panic=abort -C lto=true -C embed-bitcode=yes -C codegen-units=1 -C opt-level=z" cargo +nightly build \
    --profile release-minimal \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size,panic_immediate_abort" \
    --config="--crate-type='cdylib'" \
    --target x86_64-unknown-linux-gnu --release
