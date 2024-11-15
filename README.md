# mahjong

## 環境構築

### windows

``` sh
python -m venv .env
.env/Scripts/activate
.env/Scripts/python -m pip intall --upgrade pip
pip install maturin
```

### linux

``` sh
python -m venv .env
source .env/bin/activate
./env/bin/python -m pip intall --upgrade pip
pip install maturin
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
