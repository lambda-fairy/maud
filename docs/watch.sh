#!/bin/sh

set -e

python3 -m http.server -d site &
server_pid=$!
trap 'kill $server_pid' EXIT

while true
do
    find . -name '*.rs' -o -name '*.md' | entr -d cargo run
done
