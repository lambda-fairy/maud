#!/bin/sh

set -e

python3 -m http.server -d site &
server_pid=$!
trap 'kill $server_pid' EXIT

nproc=$(nproc || echo 4)

while true
do
    find . -name '*.rs' -o -name '*.md' -o -name '*.css' -not -path './site/*' | entr -d make -j$nproc
done
