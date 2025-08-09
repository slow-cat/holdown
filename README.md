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
wayland linux環境以外で動かないかもしれない。
