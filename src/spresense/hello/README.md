# hello

Spresense で Hello World をするためのサンプルコードと開発をするためのコンテナ環境。

## 記事

[Spresense で Hello World に苦労した](https://zenn.dev/nodamushi/articles/6897654291e7d0)

## container

コンテナ内で開発し、書き込みができるまでの環境を定義します。

[`./build_image.sh`](./build_image.sh) によってビルドできます。

Podman を使用します。

使用する Spresense SDK のバージョンを変更したい場合は [spresense_info](./spresense_info) を編集してください。

### コンテナ起動方法

[start.sh](./start.sh) を実行すると Podman のコンテナが立ち上がります。

```sh
./start.sh -d
```

ビルドだけしたい場合は`-b` で実行可能です。ビルドは [src/.bash_profile](./src/.bash_profile) の `build` 関数に定義されています。

```sh
./start.sh -b
```

`src/dist` にビルド成果物が格納されます。

## src

[src](./src) に Spresens のアプリケーションを格納しています。

このディレクトリはコンテナ内では `/work` にマウントされます。
