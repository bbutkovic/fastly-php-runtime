.SUFFIXES:

debug ?=

ifdef debug
  release :=
  target :=debug
  PHP_DEBUG :=1
else
  release :=--release
  target :=release
endif

numjobs ?=
numjobs_flag ?=
ifdef numjobs
  numjobs_flag :=-j${numjobs}
endif

$(info in $(target) mode)

PHP_WASI_SDK ?=/opt/wasi-sdk
PHP_WASI_SDK_SYSROOT :=${PHP_WASI_SDK}/share/wasi-sysroot
PHP_WASI_LIBCLANG_RT_PATH :=${PHP_WASI_SDK}/lib/clang/15.0.7/lib/wasi
PHP_WASI_EMULATORS_PATH :=${PHP_WASI_SDK}/share/wasi-sysroot/lib/wasm32-wasi
PHP_SRC_ROOT :=$(PWD)/runtime/deps/php
PHP_INCLUDES :=${PHP_SRC_ROOT},$\
  ${PHP_SRC_ROOT}/main,$\
  ${PHP_SRC_ROOT}/TSRM,$\
  ${PHP_SRC_ROOT}/Zend,$\
  ${PHP_SRC_ROOT}/ext,$\
  ${PHP_SRC_ROOT}/ext/date/lib
PHP_DEFINES :=_WASI_EMULATED_GETPID,$\
  _WASI_EMULATED_SIGNAL,$\
  _WASI_EMULATED_PROCESS_CLOCKS,$\
  _POSIX_SOURCE=1,$\
  _GNU_SOURCE=1,$\
  HAVE_FORK=0,$\
  WASM_WASI
CLANG_PATH :=${PHP_WASI_SDK}/bin/clang
CC :=${CLANG_PATH}
CFLAGS :=\
  -O3 \
  -D_WASI_EMULATED_GETPID \
  -D_WASI_EMULATED_SIGNAL \
  -D_WASI_EMULATED_PROCESS_CLOCKS \
  -D_POSIX_SOURCE=1 \
  -D_GNU_SOURCE=1 \
  -DHAVE_FORK=0 \
  -DWASM_WASI \
  -fPIC \
  -static \
  --sysroot=${PHP_WASI_SDK_SYSROOT}

ifdef debug
CFLAGS := $(CFLAGS) -g
endif

LDFLAGS :=\
  -lwasi-emulated-getpid \
  -lwasi-emulated-signal \
  -lwasi-emulated-process-clocks \
  --sysroot=${PHP_WASI_SDK_SYSROOT}
LIBPHP_CFLAGS := --rtlib=compiler-rt -static
RANLIB := ${PHP_WASI_SDK}/bin/llvm-ranlib
AR := ${PHP_WASI_SDK}/bin/llvm-ar
NM := ${PHP_WASI_SDK}/bin/llvm-nm

OPT_LEVEL ?=3

# todo: clean up this mess (:
runtime.wasm: export PHP_WASI_SDK :=${PHP_WASI_SDK}
runtime.wasm: export PHP_WASI_SDK_SYSROOT :=${PHP_WASI_SDK_SYSROOT}
runtime.wasm: export PHP_LIBCLANG_RT_PATH :=${PHP_WASI_LIBCLANG_RT_PATH}
runtime.wasm: export PHP_WASI_EMULATORS_PATH :=${PHP_WASI_EMULATORS_PATH}
runtime.wasm: export CLANG_PATH :=${CLANG_PATH}
runtime.wasm: export CC :=${CC}
runtime.wasm: export OPT_LEVEL :=${OPT_LEVEL}
runtime.wasm: export PHP_DEBUG :=${PHP_DEBUG}
runtime.wasm: export ZEND_EXTRA_LIBS :=fastlyce
runtime.wasm: export PHP_SRC_ROOT :=${PHP_SRC_ROOT}
runtime.wasm: export PHP_INCLUDES :=${PHP_INCLUDES}
runtime.wasm: export PHP_DEFINES :=${PHP_DEFINES}
runtime.wasm: export PHP_LIBPHP_PATH :=${PHP_SRC_ROOT}/libs
runtime.wasm: export PHP_CONFIGURE_FROM_ENV :=true
runtime.wasm: export PHP_PHP_API :=20210902
runtime.wasm: export PHP_DEBUG_BUILD :=no
runtime.wasm: export PHP_THREAD_SAFETY :=disabled
runtime.wasm: runtime/deps/php/libs/libphp.a
	cd runtime && cargo build $(release) && cp target/wasm32-wasi/$(target)/fastly-php-runtime.wasm ../runtime.wasm

runtime.wat: runtime.wasm
	wasm2wat runtime.wasm > runtime.wat

runtime/deps/php:
	cd runtime && mkdir -p deps && cp -r vendor/php deps/php

runtime/deps/php/Makefile: export CFLAGS :=${CFLAGS}
runtime/deps/php/Makefile: export LDFLAGS :=${LDFLAGS}
runtime/deps/php/Makefile: export CC := ${CC}
runtime/deps/php/Makefile: export RANLIB := ${RANLIB}
runtime/deps/php/Makefile: export AR := ${AR}
runtime/deps/php/Makefile: export NM := ${NM}
runtime/deps/php/Makefile: | runtime/deps/php
	cd runtime/deps/php && \
    ./buildconf --force && \
    ./configure \
    --enable-embed=static \
    --host=wasm32-wasi \
    --target=wasm32-wasi \
    --prefix=$(PWD) \
    --without-libxml \
    --disable-dom \
    --without-iconv \
    --without-openssl \
    --disable-simplexml \
    --disable-xml \
    --disable-xmlreader \
    --disable-xmlwriter \
    --without-pear \
    --disable-opcache \
    --disable-zend-signals \
    --without-pcre-jit \
    --without-sqlite3 \
    --without-pdo-sqlite \
    --enable-phar=static \
    --enable-pdo=static \
    --with-pic

runtime/deps/php/libs/libphp.a: export CFLAGS :=${CFLAGS}
runtime/deps/php/libs/libphp.a: export LDFLAGS :=${LDFLAGS}
runtime/deps/php/libs/libphp.a: export CC := ${CC}
runtime/deps/php/libs/libphp.a: export RANLIB := ${RANLIB}
runtime/deps/php/libs/libphp.a: export AR := ${AR}
runtime/deps/php/libs/libphp.a: export NM := ${NM}
runtime/deps/php/libs/libphp.a: runtime/deps/php/Makefile
	cd runtime/deps/php && make ${numjobs_flag} libphp.la

.PHONY: clean cargo-clean
clean:
	@rm -rf runtime.wasm runtime/deps/

cargo-clean: clean
	cd runtime && cargo clean