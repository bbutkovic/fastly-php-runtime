<?php

use FastlyCE\Request;

try {
    $response = Request::get("https://www.fastly.com/")
        ->send("backend_a");

    echo $response->intoBodyStr();
} catch (\Exception $e) {
    var_dump($e);
}