#!/bin/bash

python py.py $1 > rust/fosite/input.json
cd rust/fosite
cargo run