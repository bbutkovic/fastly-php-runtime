#!/usr/bin/env bash

set -euo pipefail

php_version=${PHP_VERSION:-8.2.1}

echo "Cleaning up sources and dependencies..."

rm -rf runtime/dependencies/sources/php-${php_version}

echo "Downloading PHP ${php_version}..."

git clone --depth 1 -b php-${php_version} https://github.com/php/php-src.git "runtime/dependencies/sources/php-${php_version}"
