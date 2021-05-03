#!/bin/bash

# Used for manually sending a message to a running network.
#
# You could for example spin up a full network using the Docker Compose files
# we have (to make sure the message relays are running), but remove the message
# generator service. From there you may submit messages manually using this script.

MILLAU_PORT="${MILLAU_PORT:-9945}"

case "$1" in
	remark)
		RUST_LOG=runtime=trace,substrate-relay=trace,bridge=trace \
		./target/debug/substrate-relay send-message MillauToTemplate \
			--source-host localhost \
			--source-port $MILLAU_PORT \
			--source-signer //Dave \
			--target-signer //Dave \
			--lane 00000000 \
			--origin Target \
			remark \
		;;
	transfer)
		RUST_LOG=runtime=trace,substrate-relay=trace,bridge=trace \
		./target/debug/substrate-relay send-message MillauToTemplate \
			--source-host localhost \
			--source-port $MILLAU_PORT \
			--source-signer //Dave \
			--target-signer //Dave \
			--lane 00000000 \
			--origin Target \
			transfer \
 			--amount 1000000000 \
			--recipient 6ztG3jPnJTwgZnnYsgCDXbbQVR82M96hBZtPvkN56A9668ZC \
		;;
	*) echo "A message type is require. Supported messages: remark, transfer."; exit 1;;
esac
