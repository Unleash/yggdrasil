package main

import (
	"encoding/json"
	"io/ioutil"
	"testing"
)

func TestTakeState(t *testing.T) {
	filePath := "../test-data/simple.json"
	jsonData, err := ioutil.ReadFile(filePath)
	if err != nil {
		t.Fatalf("Failed to load JSON file: %v", err)
	}

	engine := NewUnleashEngine()
	engine.TakeState(string(jsonData))
}

func TestIsEnabled(t *testing.T) {
	filePath := "../test-data/simple.json"
	jsonData, err := ioutil.ReadFile(filePath)
	if err != nil {
		t.Fatalf("Failed to load JSON file: %v", err)
	}

	var data map[string]interface{}
	err = json.Unmarshal(jsonData, &data)
	if err != nil {
		t.Fatalf("Failed to parse JSON: %v", err)
	}

	jsonString, err := json.Marshal(data)
	if err != nil {
		t.Fatalf("Failed to convert JSON to string: %v", err)
	}

	engine := NewUnleashEngine()
	engine.TakeState(string(jsonString))

	context := NewContext(
		nil,
		nil,
		nil,
		nil,
		nil,
		nil,
		nil,
	)

	isEnabled := engine.IsEnabled("Feature.A", context)
	if !isEnabled {
		t.Fatalf("Feature.A should be enabled")
	}
}

func TestGetVariant(t *testing.T) {
	filePath := "../test-data/simple.json"
	jsonData, err := ioutil.ReadFile(filePath)
	if err != nil {
		t.Fatalf("Failed to load JSON file: %v", err)
	}

	var data map[string]interface{}
	err = json.Unmarshal(jsonData, &data)
	if err != nil {
		t.Fatalf("Failed to parse JSON: %v", err)
	}

	jsonString, err := json.Marshal(data)
	if err != nil {
		t.Fatalf("Failed to convert JSON to string: %v", err)
	}

	engine := NewUnleashEngine()
	engine.TakeState(string(jsonString))

	context := NewContext(
		nil,
		nil,
		nil,
		nil,
		nil,
		nil,
		nil,
	)

	variant := engine.GetVariant("Feature.A", context)

	if variant == nil {
		variant = &VariantDef{"disabled", nil, false, engine.IsEnabled("Feature.A", context)}
	}
	if variant.Name != "disabled" {
		t.Fatalf("Feature.A should have been disabled")
	}
	if variant.Enabled {
		t.Fatalf("Feature.A should have been disabled")
	}
}

func getNullableStringProperty(jsonMap map[string]interface{}, propertyName string) *string {
	propertyValue, ok := jsonMap[propertyName]
	if !ok {
		return nil
	}

	strValue, ok := propertyValue.(string)
	if !ok {
		return nil
	}

	return &strValue
}

func getNullableMapProperty(jsonMap map[string]interface{}, propertyName string) *map[string]string {
	propertyValue, ok := jsonMap[propertyName]
	if !ok {
		return nil
	}

	interfaceMap, ok := propertyValue.(map[string]interface{})
	if !ok {
		return nil
	}

	stringMap := make(map[string]string)
	for key, value := range interfaceMap {
		stringValue, ok := value.(string)
		if ok {
			stringMap[key] = stringValue
		}
	}

	return &stringMap
}

func TestClientSpecification(t *testing.T) {
	engine := NewUnleashEngine()

	indexFilePath := "../client-specification/specifications/index.json"
	indexData, err := ioutil.ReadFile(indexFilePath)
	if err != nil {
		t.Fatalf("Failed to load JSON file: %v", err)
	}

	var testSuites []string
	err = json.Unmarshal(indexData, &testSuites)
	if err != nil {
		t.Fatalf("Failed to parse JSON: %v", err)
	}

	for _, suite := range testSuites {
		suitePath := "../client-specification/specifications/" + suite
		suiteDataBytes, err := ioutil.ReadFile(suitePath)
		if err != nil {
			t.Fatalf("Failed to load JSON file: %v", err)
		}

		var suiteData map[string]interface{}
		err = json.Unmarshal(suiteDataBytes, &suiteData)
		if err != nil {
			t.Fatalf("Failed to parse JSON: %v", err)
		}

		stateData, err := json.Marshal(suiteData["state"])
		if err != nil {
			t.Fatalf("Failed to convert state data to JSON: %v", err)
		}

		engine.TakeState(string(stateData))

		if tests, exists := suiteData["tests"]; exists {
			for _, test := range tests.([]interface{}) {
				testMap := test.(map[string]interface{})
				toggleName := testMap["toggleName"].(string)
				contextJSON := testMap["context"].(map[string]interface{})
				expectedResult := testMap["expectedResult"].(bool)

				context := NewContext(
					getNullableStringProperty(contextJSON, "userId"),
					getNullableStringProperty(contextJSON, "sessionId"),
					getNullableStringProperty(contextJSON, "environment"),
					getNullableStringProperty(contextJSON, "appName"),
					getNullableStringProperty(contextJSON, "currentTime"),
					getNullableStringProperty(contextJSON, "remoteAddress"),
					getNullableMapProperty(contextJSON, "properties"),
				)

				jsonContext, _ := json.Marshal(context)

				result := engine.IsEnabled(toggleName, context)
				if result != expectedResult {
					t.Fatalf("Failed test '%s': expected %v, got %v on context %s", testMap["description"], expectedResult, result, jsonContext)
				}
			}
		}

		if variantTests, exists := suiteData["variantTests"]; exists {
			for _, test := range variantTests.([]interface{}) {
				testMap := test.(map[string]interface{})
				toggleName := testMap["toggleName"].(string)
				contextJSON := testMap["context"].(map[string]interface{})
				expectedResult := testMap["expectedResult"].(map[string]interface{})
				context := NewContext(
					getNullableStringProperty(contextJSON, "userId"),
					getNullableStringProperty(contextJSON, "sessionId"),
					getNullableStringProperty(contextJSON, "environment"),
					getNullableStringProperty(contextJSON, "appName"),
					getNullableStringProperty(contextJSON, "currentTime"),
					getNullableStringProperty(contextJSON, "remoteAddress"),
					getNullableMapProperty(contextJSON, "properties"),
				)

				result := engine.GetVariant(toggleName, context)

				if result == nil {
					result = &VariantDef{"disabled", nil, false, false}
				}

				jsonExpectedResult, _ := json.Marshal(expectedResult)

				expectedVariant := &VariantDef{}
				err := json.Unmarshal(jsonExpectedResult, expectedVariant)
				if err != nil {
					t.Fatalf("Failed to unmarshal expectedResult into VariantDef: %v", err)
				}

				if result.Name != expectedVariant.Name {
					t.Fatalf("Failed variant test '%s': expected variant name %s, got %s", testMap["description"], expectedVariant.Name, result.Name)
				}
				if result.Enabled != expectedVariant.Enabled {
					t.Fatalf("Failed variant test '%s': expected variant enabled %v, got %v", testMap["description"], expectedVariant.Enabled, result.Enabled)
				}
				if result.Payload != nil && expectedVariant.Payload != nil {
					if result.Payload.Value != expectedVariant.Payload.Value {
						t.Fatalf("Failed variant test '%s': expected variant payload %s, got %s", testMap["description"], expectedVariant.Payload, result.Payload)
					}
					if result.Payload.Value != expectedVariant.Payload.Value {
						t.Fatalf("Failed variant test '%s': expected variant payload %s, got %s", testMap["description"], expectedVariant.Payload, result.Payload)
					}

				}
			}
		}
	}
}
