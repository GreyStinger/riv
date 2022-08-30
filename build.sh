cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu --release
cargo build --release

# Default for my screen size but change to fit yours if you are using wayland
export SCREEN_SIZE=1920,1080

upx --best --lzma "target/release/riv"
upx --best --lzma "target/x86_64-unknown-linux-gnu/release/riv"
