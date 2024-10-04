# podman の学習

Podman を使ってコンテナ間で通信してみる。

- [記事1](https://zenn.dev/nodamushi/articles/1ccd0395e2fb29)
- [記事2](https://zenn.dev/nodamushi/articles/7e356a9d81c740)

## Directory

- [client](./client): SSH Client をインストールした Ubuntu 22.04 イメージ. [id_rsa](./client/id_rsa) はあくまで実験用途であり、使用しないでください。
- [server](./server): SSH Server をインストールした Ubuntu 22.04 イメージ. [id_rsa.pub](./server/id_rsa.pub) はあくまで実験用途であり、使用しないでください。
- [docker-compose](./docker-compose/): Podman で docker compose を使ってみるサンプル

