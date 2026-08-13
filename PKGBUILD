pkgname=rscashm
pkgver=0.1.0
pkgrel=1
pkgdesc="Rust Simple Cash Manager"
arch=('x86_64')
url="https://github.com/kebinoktav/rscashm"
license=('MPL-2.0')

depends=('gcc-libs')

source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"

    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"

    install -Dm755 \
        target/release/rscashm \
        "$pkgdir/usr/bin/rscashm"

    install -Dm644 \
        rscashm.desktop \
        "$pkgdir/usr/share/applications/rscashm.desktop"

    install -Dm644 \
        rscashm.png \
        "$pkgdir/usr/share/icons/hicolor/128x128/apps/rscashm.png"
}