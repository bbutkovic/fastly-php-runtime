<?php
$bundle = new Phar('webapp.phar');
$bundle->startBuffering();

// todo: glob files

$bundle->setStub('<?php
Phar::interceptFileFuncs();
Phar::webPhar();
echo "I am intended to be executed from the Fastly CE runtime: " . php_sapi_name() . "\n";
exit -1;
__HALT_COMPILER();
');

$bundle->stopBuffering();