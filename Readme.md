# Rust Nintendo LZSS Decompressor

[![Build Status](https://travis-ci.org/silentdragonz/rust_lzss.svg?branch=master)](https://travis-ci.org/silentdragonz/rust_lzss) 
[![codecov](https://codecov.io/gh/silentdragonz/rust_lzss/branch/master/graph/badge.svg)](https://codecov.io/gh/silentdragonz/rust_lzss)

## Usage
Add to your `Cargo.toml` file:
```toml
[dependencies]
rust_lzss = "~0.1"
```

### Example
```rust
extern crate rust_lzss;

use rust_lzss::decompress;
use std::io::Cursor;

let lzss10: [u8; 11] = [ 0x10, 0x14, 0x00, 0x00, 0x08, 0x61, 0x62, 0x63, 0x64, 0xD0, 0x03, ];
let decoded = decompress(&mut Cursor::new(lzss10));
```