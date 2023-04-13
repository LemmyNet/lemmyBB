#!/bin/bash
set -ex

new_tag="$1"

old_tag=$(head -3 Cargo.toml | tail -1 | cut -d'"' -f 2)
sed -i "0,/version = \"$old_tag\"/s//version = \"$new_tag\"/g" Cargo.toml

cd lemmybb-translations
git checkout main
git pull http://weblate.join-lemmy.org/git/lemmy/lemmybb/ main --rebase
cd ..
git add lemmybb-translations

git submodule update --recursive --remote -- lemmy-translations
git add lemmy-translations

git add Cargo.toml Cargo.lock
git commit -m "Update to $new_tag"
git tag "$new_tag"
git push origin main --tags
