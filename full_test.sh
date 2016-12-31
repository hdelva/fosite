#!/bin/bash

python input.py > rust/fosite/input.json
cd rust/fosite
cargo run