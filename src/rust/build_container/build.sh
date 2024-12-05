#!/bin/bash -eu
readonly IMAGE=rust:latest

# バイナリを bin に入れたいなら install のほうが楽っぽい
# build_opt=(install --root /build --path .)
build_opt=(build --release)

dir=$(dirname "$0")
dir=$(readlink -f "$dir")

# 引数でビルドディレクトリを変更できるように
build_dir=
if [ $# -ge 1 ];then
  build_dir=$(readlink -f "$1")
else
  # target はrust analyzerやら普通にcargo build したときやらで使われるので避けた
  build_dir=$dir/build
fi
mkdir -p "$build_dir"

podman run --rm -it \
    --mount "type=bind,src=$dir,dst=$dir,ro=true" \
    --mount "type=bind,src=$build_dir,dst=/build,rw=true" \
    --workdir "$dir" \
    -e CARGO_HOME=/build/.cargo \
    -e CARGO_TARGET_DIR=/build/target \
    $IMAGE cargo "${build_opt[@]}"
