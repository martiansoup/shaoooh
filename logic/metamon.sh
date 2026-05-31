#!/bin/bash

EXTRA_ARGS=" --release"

cargo run --bin metamon$EXTRA_ARGS -- "$@"
