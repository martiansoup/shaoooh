#!/bin/bash

HOSTNAME=$(hostname)
EXTRA_ARGS=" --release"

cargo run --bin $HOSTNAME$EXTRA_ARGS -- "$@"
