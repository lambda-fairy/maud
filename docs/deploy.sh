#!/bin/sh

set -e

nproc=$(nproc || echo 4)

make clean
make -j$nproc

cd site

echo maud.lambda.xyz > CNAME

git init
git add .
git commit -m 'Deploy'

git remote add github git@github.com:lambda-fairy/maud.git
git push -f github master:gh-pages
