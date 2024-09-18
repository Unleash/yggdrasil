# Wasm Engine

## Introduction

This project is a simple WASM cross compilation of the Yggdrasil engine.

Currently, this only supports direct access to the Yggdrasil rule grammar, and is not a full implementation of the Unleash logic.

Please note that this is an experimental project and the API is subject to change rapidly as we iterate on the ideas here.

## Usage

First, install the package:

```sh
$ yarn add @unleash/yggdrasil-engine
```

Then, you can use it in your code:

```ts
import yggdrasil from '../pkg/yggdrasil_engine'

const context = {
  userId: '7'
}

const ruleEnabled = yggdrasil.evaluate('user_id > 6', context) //returns true
const ruleEnabled = yggdrasil.evaluate('user_id > 8', context) //returns false
const ruleEnabled = yggdrasil.evaluate('some rule that is nonsense', context) //raises an error
```

Rule fragments that are passed to the evaluate function must be valid Yggdrasil rules; rules that are invalid will raise an error. Valid rules will always result in a boolean value when evaluated.

### Context properties

Currently the context is built to match the [Unleash Context](https://docs.getunleash.io/reference/unleash-context),
so the special properties that are supported are:

| Property Name | Use in Unleash                         |
| ------------- | -------------------------------------- |
| environment   | the environment the app is running in  |
| userId        | an identifier for the current user     |
| sessionId     | identifier for the current session     |
| remoteAddress | the app's IP address                   |
| currentTime   | the current time in ISO format         |
| properties    | a key-value store of any data you want |

You don't have to use any of these if they have no meaning to you, using the properties object is the most flexible way to pass data into the engine but it does mean the rules you need to produce are slightly more verbose:

```js
const context = {
  properties: {
    customProperty: '7'
  }
}

const result = yggdrasil.evaluate('context["customProperty"] > 6', context) // matches the "customProperty" field on the context and returns true
```

Please note that you **must** pass a context object, even if it is empty. Failure to do so will result in an error being raised by the engine.

## Development

This project uses wasm-bindgen to generate the Rust/JS bindings. To build the
project, run:

```sh
$ wasm-pack build --target nodejs
```

There's also a set of integration tests in the `e2e-tests` directory, which will ensure that the WASM module can be loaded and used in Node JS and that calls to the engine are correctly managed. These must be run within the e2e-tests directory:

```sh
$ cd e2e-tests
$ bun i
$ bun test
```
