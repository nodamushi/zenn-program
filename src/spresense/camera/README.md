# camera

Spresense のカメラを試す。

開発環境のコンテナは [hello](../hello/) で作成したものをそのまま流用します。

### コンテナ起動方法

[start.sh](./start.sh) を実行すると Podman のコンテナが立ち上がります。

> [!IMPORTANT]
> [hello](../hello/) であらかじめイメージをビルドしておいてください

```sh
./start.sh -d
```

ビルドだけしたい場合は`-b` で実行可能です。ビルドは [src/.bash_profile](./src/.bash_profile) の `build` 関数に定義されています。

```sh
./start.sh -b
```

`src/dist` にビルド成果物が格納されます。

ビルドして書き込みまで行うには以下を実行してください。

```sh
./start.sh --deploy
```

## src

[src](./src) に Spresens のアプリケーションを格納しています。

このディレクトリはコンテナ内では `/work` にマウントされます。


### src/include

[src/include](./src/include/) にライブラリ化したカメラ操作、USB Serial 操作クラスがあります。

## 実行方法（拡張ボードあり）

Spresense の拡張ボードがある場合は、拡張ボードの USB も PC に繋いでおいてください。

Spresense に書き込んで、接続し、 `sercon` を実行するとホストPCから更に一つシリアルポートが見えるようになります。

```sh
nsh> sercon
```

Windows 等の場合は Teraterm などで新たに見えるようになったシリアルポートに ボーレート 30000000 で接続しておきます。

接続したらアプリを実行してください。

```sh
nsh> cam usb
```

撮影したJPEG画像が Base64 変換されて表示されます。

## 実行方法（拡張ボードなし）

Spresense の拡張ボードがなくても動作します。ただし、出力にだいぶ時間がかかります。

```sh
nsh> cam
```

撮影したJPEG画像が Base64 変換されて表示されます。
