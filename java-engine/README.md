# Java Bindings to Yggdrasil

This project provides a pure Java wrapper for the Yggdrasil engine, built on top of a compiled [WebAssembly core](../pure-wasm). It requires **no native libraries or JNI**, and runs entirely within the JVM using a WASM runtime.

## ☕ Overview

The Java Engine embeds the Yggdrasil WASM core using [Chicory](https://github.com/dylibso/chicory), providing full engine functionality with no external dependencies. It enables evaluation of feature toggles, variants, and gradual rollouts directly from Java code.

## Usage

The `UnleashEngine` class provides a pure Java wrapper around the Yggdrasil feature evaluation engine. It allows you to evaluate feature toggles and variants entirely in-memory, without calling back to a remote Unleash server at runtime. This is intended to be used as part of a larger project that's capable of fetching the feature toggle definitions.

### 📥 Loading State

Before evaluating any features, you must initialize the engine with feature toggle definitions. This is done using the `takeState` method. The input to takeState should be the raw JSON response from the Unleash `/api/client/features endpoint`. For example:

``` bash
curl http://unleash-url/api/client/features -H "Authorization: YOUR_API_TOKEN" > toggles.json
```

``` java
Path path = Path.of("toggles.json");

String clientFeaturesJson = Files.readString(path);

UnleashEngine engine = new UnleashEngine();
engine.takeState(clientFeaturesJson);

```

### Querying Toggle State

Once the engine is initialized, you can evaluate toggles using the isEnabled or getVariant methods:

``` java
Context context = new Context();
context.setUserId("user-123");

WasmResponse<Boolean> enabledResponse = engine.isEnabled("some-toggle", context);

if (Boolean.TRUE.equals(enabledResponse.getValue())) {
    // Feature is enabled for this context
}

if(enabledResponse.impressionData) {
    // Impression data has been enabled in Unleash
}

WasmResponse<VariantDef> response = this.featureRepository.getVariant("some-toggle-with-variants", context);

VariantDef variant = response.getValue();
if (variantResult.getValue() != null) {
    // do something with the variant
}
```

You can also query a list of toggles that's the engine currently knows about:

``` java
List<FeatureDef> toggles = engine.listKnownToggles();
for (FeatureDef toggle : toggles) {
    System.out.println("Toggle: " + toggle.getName());
}
```

## Metrics

Metrics are automatically collected through the isEnabled/getVariant calls. The metrics can be queried back like so:

``` java
MetricsBucket metrics = engine.getMetrics();
```

This will clear the current metrics buffer. This means that if the caller attempts to send this upstream and that call fails, the caller is responsible for retrying.


## Metadata Methods

The engine provides a few methods to retrieve some static metadata about what it supports.

You can query the version of the underlying Yggdrasil engine, which will return a semver string:

``` java
String version = UnleashEngine.getCoreVersion(); //1.2.1
```

You can also retrieve the list of built in strategies that the engine is aware of:

``` java
List<String> strategies = UnleashEngine.getBuiltInStrategies();
```


## Development

### Prerequisites

To work with this project, you’ll need:

- The Yggdrasil WASM binary (compiled with Rust)
- [flatc version 25.2.10](https://github.com/google/flatbuffers/releases/tag/v25.2.10) for regenerating the Java FlatBuffer bindings

### Linting

```bash
./gradlew spotlessApply
```

### Testing

The WASM library is automatically built on running tests but you'll need to make sure that you've set up the [WASM build correctly](../pure-wasm/README.md). We use Gradle here, tests can be invoked with:

``` bash
./gradlew test
```

### FlatBuffer Bindings

The Java engine uses FlatBuffers for communication with the WASM core. If you make changes to the data interchange format, regenerate the bindings like this:

``` bash
flatc --java -o java-engine/src/main/java flat-buffer-defs/enabled-message.fbs

```

You'll need to update the [WASM](../pure-wasm/) code as well to handle any changes you make to the FlatBuffer definitions.

### Building a JAR for Testing

You may want to build a JAR for linking to a local project for testing. To do this, make sure you've followed the [instructions for building](../pure-wasm/README.md) the WASM code.

You can then run the local publish

``` bash
./gradlew publishToMavenLocal
```

This will build a JAR and make it available in your local maven repository

