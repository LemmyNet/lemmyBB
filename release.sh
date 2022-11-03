#!/bin/bash
set -ex

new_tag="$1"

old_tag=$(head -3 Cargo.toml | tail -1 | cut -d'"' -f 2)
sed -i "0,/version = \"$old_tag\"/s//version = \"$new_tag\"/g" Cargo.toml

git add Cargo.toml
git commit -m "Update to $new_tag"
git tag "$new_tag"
git push --tags

sudo docker build . -f docker/Dockerfile -t "lemmynet/lemmybb:$new_tag"
sudo docker push "lemmynet/lemmybb:$new_tag"
