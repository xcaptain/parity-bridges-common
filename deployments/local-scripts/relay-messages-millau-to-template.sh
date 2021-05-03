#!/bin/bash
# A script for relaying Millau messages to the Template chain.
#
# Will not work unless both the Template and Millau are running (see `run-template-node.sh`
# and `run-millau-node.sh).
set -xeu

MILLAU_PORT="${MILLAU_PORT:-9945}"
TEMPLATE_PORT="${TEMPLATE_PORT:-9944}"

RUST_LOG=bridge=trace \
./target/debug/substrate-relay relay-messages MillauToTemplate \
	--lane 00000000 \
	--source-host localhost \
	--source-port $MILLAU_PORT \
	--source-signer //Bob \
	--target-host localhost \
	--target-port $TEMPLATE_PORT \
	--target-signer //Bob \
	--prometheus-host=0.0.0.0
