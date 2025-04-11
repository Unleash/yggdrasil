You'll need to generate the flat buffer defs if you need them. You need flatc version 23.1.21 or below to generate valid Rust.

Generate them with this:

`flatc --rust -o pure-wasm/src flat-buffer-defs/enabled-message.fbs`
`flatc --rust -o pure-wasm/src flat-buffer-defs/enabled-response.fbs`