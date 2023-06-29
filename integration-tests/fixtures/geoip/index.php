<?php

$geoipResult = \FastlyCE\Geo::lookup('123.123.123.123');

echo $geoipResult->city;