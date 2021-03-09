You'll need Rust and Cargo to run and build the compiler. I'd personally recommend rustup:
https://rustup.rs

You can run the compiler directly with cargo (assuming you're running from the base directory):
`cargo run example/testing.ice`

Or build libslice and the slicec-cs binary, and use them directly:
`cargo build --release`
`target/release/slicec-cs example/testing.ice`
