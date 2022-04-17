cargo run --release --example generate -- generated.rwy
# hexdump generated.rwy
cargo run --release --features=zeno --example to_png -- generated
