# src (/work)

Spresense の開発を行うディレクトリ。このディレクトリが approot となる。

コンテナ内では /work にマウントされます。

## コンテナ内でのビルド方法

```sh
build
```

[.bash_profile](./.bash_profile) に定義されています。

## アプリ

- [my_hello](./my_hello/): 単なる Hello World

## その他のファイル

- [compile_flags.txt](./compile_flags.txt): clangd で補完を出すための設定ファイル.ビルドに関係なし
- [.editorconfig](./.editorconfitg): Editor Config
- [.sdksubdir](./sdksubdir): 勝手に作られた。approot であることのマーカーかな
- [Makefile](./Makefile): 勝手に作られた。直接は実行できない Makefile

