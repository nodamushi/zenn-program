# Rust で unzip

Rust で Unzip するテスト。

なお、zip クレートの最新版は 2025/4月現在 2.6.1 ですが Ripunzip (2.0.1) は対応してないため 2.3 に固定しています。（2.6.1と速度差は特に見られなかったので問題ないでしょう。）

## 結果

```
[LOG] Test unzip::ZipExtra
[LOG]   Result: 559.948041ms
[LOG] Test unzip::Ripunzip
[LOG]   Result: 221.691774ms
[LOG] Test unzip::AsyncZip
[LOG]   Result: 1.7914359s
[LOG] Test unzip::AsyncZipParallel
[LOG]  cores = 16
[LOG]   Result: 510.305335ms
```

- CPU: AMD Ryzen 7 5800X 8-Core Processor
- WSL2: Ubntu 24.04
- SSD: Nextorage SSD NEM-PA2TB
