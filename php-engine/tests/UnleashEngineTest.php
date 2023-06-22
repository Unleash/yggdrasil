<?php

use PHPUnit\Framework\TestCase;
use Unleash\Yggdrasil\UnleashEngine;
use Unleash\Yggdrasil\Context;


final class UnleashEngineTest extends TestCase
{
    public function testIsEnabledWithNothing()
    {
        $engine = new UnleashEngine();
        $context = new Context("test", "test", "test", "test", "test", "test", new stdClass());

        $is_enabled = $engine->isEnabled("test", $context);

        $this->assertFalse($is_enabled);
    }

    public function testTakeState()
    {
        $engine = new UnleashEngine();
        $filePath = __DIR__ . '../../../test-data/simple.json';
        $jsonData = file_get_contents($filePath);

        if ($jsonData === false) {
            $this->fail("Failed to load JSON file");
        }

        $result = $engine->takeState($jsonData);

        $this->assertNotNull($result);
    }

    public function testIsEnabled()
    {
        $engine = new UnleashEngine();
        $filePath = __DIR__ . '../../../test-data/simple.json';
        $jsonData = file_get_contents($filePath);

        if ($jsonData === false) {
            $this->fail("Failed to load JSON file");
        }

        $engine->takeState($jsonData);
        $context = new Context("test", "test", "test", "test", "test", "test", new stdClass());

        $is_enabled = $engine->isEnabled("test", $context);
        $this->assertFalse($is_enabled);
    }

    public function testGetVariant()
    {
        $engine = new UnleashEngine();
        $filePath = __DIR__ . '../../../test-data/simple.json';
        $jsonData = file_get_contents($filePath);

        if ($jsonData === false) {
            $this->fail("Failed to load JSON file");
        }

        $engine->takeState($jsonData);
        $context = new Context("test", "test", "test", "test", "test", "test", new stdClass());

        $variant = $engine->getVariant("test", $context);
        $this->assertNotNull($variant);

        $this->assertEquals("disabled", $variant->name);
        $this->assertFalse($variant->enabled);
    }

    function testClientSpec()
    {
        $unleashEngine = new UnleashEngine();

        $testSuites = json_decode(file_get_contents("../../client-specification/specifications/index.json"));
        foreach ($testSuites as $suite) {
            $suitePath = "../../client-specification/specifications/" . $suite;

            $suiteData = json_decode(file_get_contents($suitePath));
            $unleashEngine->takeState(json_encode($suiteData->state));

            $tests = isset($suiteData->tests) ? $suiteData->tests : [];
            foreach ($tests as $test) {
                $contextData = $test->context;
                $toggleName = $test->toggleName;
                $expectedResult = $test->expectedResult;
                $context = new Context($contextData->userId, $contextData->sessionId, $contextData->remoteAddress, $contextData->environment, $contextData->appName, $contextData->currentTime, $contextData->properties);

                $result = $unleashEngine->isEnabled($toggleName, $context);
                $this->assertEquals($result, $expectedResult, "Failed test '{$test->description}': expected {$expectedResult}, got {$result}");
            }

            $variantTests = isset($suiteData->variantTests) ? $suiteData->variantTests : [];
            foreach ($variantTests as $test) {
                $contextData = $test->context;
                $toggleName = $test->toggleName;
                $expectedResult = $test->expectedResult;
                $context = new Context($contextData->userId, $contextData->sessionId, $contextData->remoteAddress, $contextData->environment, $contextData->appName, $contextData->currentTime, $contextData->properties);

                $result = $unleashEngine->getVariant($toggleName, $context);
                $jsonResult = json_encode($result);
                $expectedJsonResult = json_encode($expectedResult);

                $this->assertEquals($jsonResult, $expectedJsonResult, "Failed test '{$test->description}': expected {$expectedJsonResult}, got {$jsonResult}");
            }
        }

    }
}