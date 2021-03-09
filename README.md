# Requirements
You'll need Rust and Cargo to run and build the compiler. I'd personally recommend rustup:
https://rustup.rs

# Usage
You can run the compiler directly with cargo (assuming you're running from the base directory):

```
cargo run example/testing.ice
```

Or build libslice and the slicec-cs binary, and use them directly:

```
cargo build --release
target/release/slicec-cs example/testing.ice
```

# Examples
There are 3 example files, one with no errors that generated code:
```
target/release/slicec-cs example/testing.ice
```
one that showcases a syntax error, when the parser can't parse the input file and dies:
```
target/release/slicec-cs example/syntaxerrors.ice
```
and another which shows a semantic error, where the input can be parsed, but there's an error in the user's definitions:
```
target/release/slicec-cs example/sliceerrors.ice
```
The compiler doesn't support passing directores, but does support multiple files being passed in. For instance, running:
```
target/release/slicec-cs example/testing.ice example/sliceerrors.ice
```
will show a redefinition conflict between the 2 slice files.
