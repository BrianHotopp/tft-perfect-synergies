#!/bin/bash
cargo build --target x86_64-pc-windows-gnu --release
cargo build --target x86_64-unknown-linux-musl --release


