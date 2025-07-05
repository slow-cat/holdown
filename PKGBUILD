pkgname="holdown"
pkgver=0.1.0
pkgrel=1
pkgdesc="Support the 'right hold down' function on laptops as well"
arch=('x86_64')
url="https://github.com/slow-cat/holdown"
license=('MIT' 'Apache')
depends=()
makedepends=('rust' 'cargo' 'git')
source=("$pkgname::git+$url.git#tag=v$pkgver")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$pkgname"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$pkgname"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
}

