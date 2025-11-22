#!/bin/bash

HOSTNAME=$(hostname)
EXTRA_ARGS=""

if [[ "$HOSTNAME" == "bishaan" ]]; then
  EXTRA_ARGS+=" --release"
fi

cargo run --bin $HOSTNAME$EXTRA_ARGS -- "$@"