<?php

namespace Fastly\PhpRuntime\Cli;

use Exception;
use Symfony\Component\Console\Application;
use Fastly\PhpRuntime\Cli\Commands\BundleCommand;

class Tool {

    /**
     * @throws Exception
     */
    public static function run(): void
    {
        $app = new Application();
        $app->add(new BundleCommand());
        $app->run();
    }
}