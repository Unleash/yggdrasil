package main

/*
#cgo LDFLAGS: -L. -lyggdrasilffi
#include "unleash_engine.h"
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"log"
	"unsafe"
)

const emptyCustomStrategyResults = "{}"

type UnleashEngine struct {
	ptr unsafe.Pointer
}

type response[T any] struct {
	StatusCode   string `json:"status_code,omitempty"`
	Value        T      `json:"value,omitempty"`
	ErrorMessage string `json:"error_message,omitempty"`
}

func NewUnleashEngine() *UnleashEngine {
	ptr := unsafe.Pointer(C.new_engine())
	return &UnleashEngine{ptr: ptr}
}

func (e *UnleashEngine) TakeState(json string) {
	C.take_state(e.ptr, C.CString(json))
}

func (e *UnleashEngine) IsEnabled(toggleName string, context *Context) bool {
	ctoggleName := C.CString(toggleName)

	jsonContext, err := json.Marshal(context)

	if err != nil {
		log.Fatalf("Failed to serialize context: %v", err)
		return false
	}
	cjsonContext := C.CString(string(jsonContext))
	cemptyStrategyResults := C.CString(emptyCustomStrategyResults)

	defer func() {
		C.free(unsafe.Pointer(ctoggleName))
		C.free(unsafe.Pointer(cjsonContext))
		C.free(unsafe.Pointer(cemptyStrategyResults))
	}()

	cenabled := C.check_enabled(e.ptr, ctoggleName, cjsonContext, C.CString(emptyCustomStrategyResults))
	defer C.free_response(cenabled)

	jsonEnabled := C.GoString(cenabled)

	enabled := &response[bool]{}
	err = json.Unmarshal([]byte(jsonEnabled), enabled)
	if err != nil {
		fmt.Printf("Failed to deserialize enabled: %v\n", err)
		return false
	}
	return enabled.Value
}

type VariantDef struct {
	Name           string   `json:"name,omitempty"`
	Payload        *Payload `json:"payload,omitempty"`
	Enabled        bool     `json:"enabled,omitempty"`
	FeatureEnabled bool     `json:"feature_enabled,omitempty"`
}

type Payload struct {
	PayloadType string `json:"type,omitempty"`
	Value       string `json:"value,omitempty"`
}

func (e *UnleashEngine) GetVariant(toggleName string, context *Context) *VariantDef {
	ctoggleName := C.CString(toggleName)

	jsonContext, err := json.Marshal(context)

	if err != nil {
		fmt.Printf("Failed to serialize context: %v\n", err)
		return nil
	}

	cjsonContext := C.CString(string(jsonContext))
	cemptyStrategyResults := C.CString(emptyCustomStrategyResults)

	defer func() {
		C.free(unsafe.Pointer(ctoggleName))
		C.free(unsafe.Pointer(cjsonContext))
		C.free(unsafe.Pointer(cemptyStrategyResults))
	}()

	cvariantDef := C.check_variant(e.ptr, ctoggleName, cjsonContext, cemptyStrategyResults)
	defer C.free_response(cvariantDef)

	jsonVariant := C.GoString(cvariantDef)

	variantDef := &response[*VariantDef]{}
	err = json.Unmarshal([]byte(jsonVariant), variantDef)
	if err != nil {
		fmt.Printf("Failed to deserialize variantDef: %v\n", err)
		return nil
	}

	return variantDef.Value
}

type Context struct {
	UserID        *string            `json:"userId,omitempty"`
	SessionID     *string            `json:"sessionId,omitempty"`
	Environment   *string            `json:"environment,omitempty"`
	AppName       *string            `json:"appName,omitempty"`
	CurrentTime   *string            `json:"currentTime,omitempty"`
	RemoteAddress *string            `json:"remoteAddress,omitempty"`
	Properties    *map[string]string `json:"properties,omitempty"`
}

func NewContext(userID, sessionID, environment, appName, currentTime, remoteAddress *string, properties *map[string]string) *Context {
	return &Context{
		UserID:        userID,
		SessionID:     sessionID,
		Environment:   environment,
		AppName:       appName,
		CurrentTime:   currentTime,
		RemoteAddress: remoteAddress,
		Properties:    properties,
	}
}
