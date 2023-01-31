#!/usr/bin/env bash

set -euo pipefail

wasi_version=17
wasi_version_full=${wasi_version}.0

if [[ ! -d "tmp/wasi-sdk-${wasi_version_full}" ]]; then
    echo "Downloading WASI SDK..."

    wget -O tmp/wasi-sdk-${wasi_version_full}-linux.tar.gz https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${wasi_version}/wasi-sdk-${wasi_version_full}-linux.tar.gz
    tar -C tmp/ -xvf tmp/wasi-sdk-${wasi_version_full}-linux.tar.gz
fi

export WASI_SDK_ROOT=$(realpath tmp/wasi-sdk-${wasi_version_full})
export WASI_SYSROOT="${WASI_SDK_ROOT}/share/wasi-sysroot"
export CC=${WASI_SDK_ROOT}/bin/clang
export LD=${WASI_SDK_ROOT}/bin/wasm-ld
export CXX=${WASI_SDK_ROOT}/bin/clang++
export NM=${WASI_SDK_ROOT}/bin/llvm-nm
export AR=${WASI_SDK_ROOT}/bin/llvm-ar
export RANLIB=${WASI_SDK_ROOT}/bin/llvm-ranlib

export CFLAGS_CONFIG="-O2"

export CFLAGS_WASI="--sysroot=${WASI_SYSROOT} -D_WASI_EMULATED_GETPID -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_PROCESS_CLOCKS"
export LDFLAGS_WASI="--sysroot=${WASI_SYSROOT} -lwasi-emulated-getpid -lwasi-emulated-signal -lwasi-emulated-process-clocks"

export CFLAGS_PHP='-D_POSIX_SOURCE=1 -D_GNU_SOURCE=1 -DHAVE_FORK=0 -DWASM_WASI'

# We need to add LDFLAGS to CFLAGS because autoconf compiles(+links) to binary when checking stuff
export LDFLAGS="${LDFLAGS_WASI}"
export CFLAGS="${CFLAGS_CONFIG} ${CFLAGS_WASI} ${CFLAGS_PHP} ${LDFLAGS}"


# export CFLAGS_CONFIG="-O2"
# export CFLAGS="--sysroot=${WASI_SYSROOT}"
# export CXXFLAGS="--sysroot=${WASI_SYSROOT}"
# export LDFLAGS="--sysroot=${WASI_SYSROOT} -L${WASI_SYSROOT}/usr/lib"


# export CFLAGS_CLEAN="--sysroot=${WASI_SYSROOT}"
# export LDFLAGS_WASI="--sysroot=${WASI_SYSROOT}"


php_version=${PHP_VERSION:-8.2.1}

if [[ -d "runtime/dependencies/patches/php-${php_version}" ]]; then
    echo "Patching PHP ${php_version}..."

    for patch in runtime/dependencies/patches/php-${php_version}/*.patch; do
        patch --forward -p1 -d "runtime/dependencies/sources/php-${php_version}" < "$patch"
    done
fi

echo "Configuring PHP ${php_version}..."

cd "runtime/dependencies/sources/php-${php_version}"

./buildconf --force

./configure \
    --host=wasm32-wasi \
    --target=wasm32-wasi \
    --enable-embed=static \
    --oldincludedir=${WASI_SYSROOT}/usr/include \
    --disable-opcache \
    --without-libxml  \
    --without-sqlite3 \
    --without-pdo-sqlite \
    --disable-dom \
    --disable-simplexml \
    --disable-xml \
    --disable-xmlreader \
    --disable-xmlwriter

echo "Building PHP ${php_version}..."

make libphp7.la

# cp libs/libphp*.la ../../lib/libphp.la
# cp libs/libphp*.a ../../lib/libphp7.a
