#!/bin/bash
set -xeu

sleep 3
curl -v http://millau-node-alice:9933/health
curl -v https://rococo-rpc.polkadot.io:443/health

/home/user/substrate-relay init-bridge RococoToMillau \
	--source-host rococo-rpc.polkadot.io \
	--source-port 443 \
	--source-secure \
	--target-host millau-node-alice \
	--target-port 9944 \
	--target-signer //Harry

# Give chain a little bit of time to process initialization transaction
sleep 6
/home/user/substrate-relay relay-headers RococoToMillau \
	--source-host rococo-rpc.polkadot.io \
	--source-port 443 \
	--source-secure \
	--target-host millau-node-alice \
	--target-port 9944 \
	--target-signer //Harry \
	--prometheus-host=0.0.0.0

