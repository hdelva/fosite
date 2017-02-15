#!/bin/bash

python py.py > rust/fosite/input.json
cd rust/fosite
cargo run --release
