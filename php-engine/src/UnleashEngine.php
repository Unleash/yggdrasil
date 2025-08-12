<?php
namespace Unleash\Yggdrasil;

use FFI;
use stdClass;


class Context
{
    public ?string $userId;
    public ?string $sessionId;
    public ?string $remoteAddress;
    public ?string $environment;
    public ?string $appName;
    public ?string $currentTime;
    public ?stdClass $properties;

    public function __construct(?string $userId, ?string $sessionId, ?string $remoteAddress, ?string $environment, ?string $appName, ?string $currentTime, ?stdClass $properties)
    {
        $this->userId = $userId;
        $this->sessionId = $sessionId;
        $this->remoteAddress = $remoteAddress;
        $this->environment = $environment;
        $this->appName = $appName;
        $this->currentTime = $currentTime;
        $this->properties = $properties;
    }
}

class UnleashEngine
{
    private $ffi;

    public function __construct()
    {
        $ffi_def = file_get_contents(__DIR__ . "/../../yggdrasilffi/unleash_engine.h");
        $ffi_lib_path = getenv("YGGDRASIL_LIB_PATH");
        $ffi_lib = $ffi_lib_path . "/libyggdrasilffi.so";
        $this->ffi = FFI::cdef($ffi_def, $ffi_lib);
        $this->state = $this->ffi->engine_new();
    }

    public function __destruct()
    {
        $this->ffi->engine_free($this->state);
    }

    public function takeState(string $json): string
    {
        return $this->ffi->engine_take_state($this->state, $json);
    }

    public function getState(): string
    {
        $result = $this->ffi->get_state($this->state);
        if ($result == null) {
            // Shouldn't happen anymore, but return empty state as fallback
            return '{"version":2,"features":[]}';
        }
        $state_json = FFI::string($result);
        $this->ffi->free_response($result);
        return $state_json;
    }

    public function getVariant(string $toggle_name, Context $context): ?stdClass
    {
        $context_json = json_encode($context);
        $variant_json_cdata = $this->ffi->engine_get_variant($this->state, $toggle_name, $context_json);
        if ($variant_json_cdata == null) {
            return null;
        } else {
            $variant_json = FFI::string($variant_json_cdata);
            $variant = json_decode($variant_json);
            $this->ffi->engine_free_variant_def($variant_json_cdata);
            return $variant;
        }
    }

    public function isEnabled(string $toggle_name, Context $context): bool
    {
        $context_json = json_encode($context);
        return $this->ffi->engine_is_enabled($this->state, $toggle_name, $context_json);
    }
}