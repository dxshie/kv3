# kv3

A Rust crate for parsing Valve's KeyValues3 (KV3) format.

<!-- If your crate is published on crates.io, you can include these badges.
[![Crates.io](https://img.shields.io/crates/v/kv3.svg)](https://crates.io/crates/kv3)
[![Documentation](https://docs.rs/kv3/badge.svg)](https://docs.rs/kv3)
[![License](https://img.shields.io/crates/l/kv3.svg)](https://github.com/yourusername/kv3/blob/main/LICENSE)
-->

## Overview

`kv3` is a Rust library for parsing and serializing the KeyValues3 (KV3) format used by Valve in their games and tools. It allows you to read KV3 files and access their data in a structured way.

## Features

- **Parsing**: Convert KV3-formatted strings into Rust data structures with serde.
- **Support for Comments**: Handles single-line (`//`), multi-line (`/* ... */`), and XML-style (`<!-- ... -->`) comments.
- **Support for Multiline Strings**: Parses multiline strings enclosed in triple double-quotes (`"""`).
- **Handles Various Data Types**: Supports booleans, integers, floats, strings, arrays, hex arrays(binary blobs), objects, and null values.
- **Customizable Parsing**: Built using the [`nom`](https://github.com/Geal/nom) parser combinator library for flexibility.

## Installation

Add `kv3` to your `Cargo.toml` dependencies:

```toml
[dependencies]
kv3 = { git = "https://github.com/dxshie/kv3", features = ["serde"] }