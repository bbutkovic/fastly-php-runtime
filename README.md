# Fastly PHP Compute@Edge Runtime

![GitHub Workflow Status (with branch)](https://img.shields.io/github/actions/workflow/status/bbutkovic/fastly-php-runtime/runtime.yml?branch=main&style=flat-square)
![GitHub release (latest by date)](https://img.shields.io/github/downloads/bbutkovic/fastly-php-runtime/latest/total?label=Runtime%20downloads&style=flat-square)

| :exclamation: NOTICE                                                                                                                             |
|:-------------------------------------------------------------------------------------------------------------------------------------------------|
| Fastly PHP Compute@Edge runtime is currently in a pre-release state, breaking changes **will** happen and things are **not guaranteed to work**. |
| Usage in production is not recommended at the moment.                                                                                            |

## Getting Started

It is recommended to use the [PHP Runtime Tool](https://github.com/bbutkovic/fastly-php-runtime-tool).
It manages building the runtime along with your code, building it into a single WASM file that you can use with Fastly's Compute@Edge.

To get started, install the PHP Runtime Tool on your system:

| :memo: NOTICE                                                                                       |
|:----------------------------------------------------------------------------------------------------|
| You will need to make sure you have the required dependencies installed on your system.             |
| See the [Development Environment Dependencies](#development-environment-dependencies) section below.|

```console
composer global require "bbutkovic/fastly-php-runtime-tool"
```

Once installed, you can use the CLI to build your PHP code:

```console
fastly-compute-php bundle index.php
```

---

Alternatively, you can add `bbutkovic/fastly-php-runtime-tool` as a Composer dependency to your project:

```console
composer require --dev "bbutkovic/fastly-php-runtime-tool
```

The [starter kit](#starter-kit) takes care of this for you:

## Usage

Here's an example application:

```php
<?php

header("X-Test-Header: Hello, world");

echo "Looking up geolocation information" . PHP_EOL;

$geo = \FastlyCE\Geo::lookup($_SERVER['REMOTE_ADDR']);

echo "You are visiting us from " . $geo->city . PHP_EOL;
```

## PHP Stubs

Since calls to Fastly's Compute@Edge are implemented through a PHP module, it is recommended to download the PHP stubs
to your project.

```console
fastly-compute-php stubs:download
```

| :memo: NOTICE                                                                                       |
|:----------------------------------------------------------------------------------------------------|
| If you're using the fastly-compute-php tool as a Composer dependency, this may work automatically.  |

## Starter Kit

You can use the [Default PHP starter kit](https://github.com/bbutkovic/compute-starter-kit-php-default) by:

* creating a new Composer project:

    ```console
    composer create-project "bbutkovic/compute-starter-kit-php-default" --ask --stability=dev --repository='{"type": "vcs", "url": "git@github.com:bbutkovic/compute-starter-kit-php-default.git"}'
    ```

* creating a new repository from the template:
[Create a new repository from compute-starter-kit-php-default](https://github.com/bbutkovic/compute-starter-kit-php-default/generate)

* cloning the repository from GitHub:

    ```console
    git clone https://github.com/bbutkovic/compute-starter-kit-php-default.git
    ```

Once you have the code ready locally, simply install the Composer dependencies by running:

```console
composer install
```

and running it with Fastly CLI:

```console
fastly compute serve
```

## Development Environment Dependencies

The following development environment dependencies are required:

* [Composer](https://getcomposer.org/) - Used for obtaining the PHP Runtime Tool. You can download the latest version
by following the [Composer Download guide](https://getcomposer.org/download/).

* [Fastly CLI](https://developer.fastly.com/reference/cli/) - Used for running your Fastly PHP Runtime code locally and
publishing the built asset to Fastly's Compute@Edge platform. You can download the latest version by following Fastly's
[installation guide](https://developer.fastly.com/learning/tools/cli).

* [Wizer](https://github.com/bytecodealliance/wizer) - Used for bundling the PHP code. You can download it from the
[Wizer releases page on GitHub](https://github.com/bytecodealliance/wizer/releases), make sure to download
the correct build for your system and place the downloaded executable in your `$PATH`.

* [PHP 8.1](https://www.php.net/downloads.php#v8.1.17) - This is required for running the PHP Runtime Tool
on your development machine.

## License

[MIT License](LICENSE)
