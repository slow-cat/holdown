# holdown
ラップトップで右ホールドをするためのツール。Chatgptで作った。
私のラップトップでは右ホールド（右クリック長押し）に非対応だったので作成した。

## Installation

```
cargo install --path . 
```
もしくはおすすめしないがArchなら

```
makepkg -si
```

## Usage
```
sudo holdown
```
sudo権限がなければevdevに直接アクセスすることができないので必要っぽい。

## 追記
libinputのアップデートを行ったっところどうやらlibinputのデバック用ツールが分離されたようで動かなくなっていたので修正した
