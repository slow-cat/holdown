pkgname="holdown"
pkgver=0.1.1
pkgrel=1
pkgdesc="Support the 'right hold down' function on laptops as well"
arch=('x86_64')
url="https://github.com/slow-cat/holdown"
license=('MIT' 'Apache')
depends=('libinput' 'libinput-tools')
makedepends=('rust' 'cargo' 'git')
source=()
sha256sums=()

build() {
    cargo build --release
}

package() {
    install -Dm755 "../target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
}

