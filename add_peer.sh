#!/bin/bash

PEER_NAME="$1"

docker run -it --rm \
    --name $PEER_NAME \
    --network p2p_chat \
    -v "$(pwd):/app" \
    netshoot_rust



