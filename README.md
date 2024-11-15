# mahjong

## 環境構築

### windows

``` sh
python -m venv .env
.env/Scripts/activate
.env/Scripts/python -m pip intall --upgrade pip
pip intall pre-commit
pre-commit install
pip install maturin
cargo add cargo-llvm-cov
```

### linux

``` sh
python -m venv .env
source .env/bin/activate
./env/bin/python -m pip intall --upgrade pip
pip intall pre-commit
pre-commit install
pip install maturin
cargo add cargo-llvm-cov
```

## コマンド

### maturin 開発

``` sh
maturin develop
```

### maturin ビルド

``` sh
maturin build
```

## 使い方

### unit test

``` sh
cargo test
```

### coverage

``` sh
cargo llvm-cov --html
```
