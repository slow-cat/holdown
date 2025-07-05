pkgname="holdown"
pkgver=0.1.0
pkgrel=1
pkgdesc="Support the 'right hold down' function on laptops as well"
arch=('x86_64')
url="https://github.com/slow-cat/holdown"
license=('MIT' 'Apache')
depends=()
makedepends=('rust' 'cargo' 'git')
source=("git+https://github.com/slow-cat/holdown.git")
sha256sums=('SKIP')

build() {
    cd "$srcdir/holdown"
    cargo build --release
}

package() {
    install -Dm755 "$srcdir/holdown/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
}

