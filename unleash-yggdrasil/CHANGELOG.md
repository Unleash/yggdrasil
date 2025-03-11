# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.17.3 (2025-03-11)

### Chore

 - <csr-id-9096eea4dbb24d71ae4876c976f85e5aa9472b04/> javascript engine
   * chore: javascript engine
   
   * docs: update README, package.json, tests
   
   * fix: do not push back to git, its our temp pkg dir
   
   * chore: use set version instead
   
   * chore: use jq instead
   
   * chore: bump yggdrasil-wasm
   
   * fix: remove defaults from isEnabled and getVariant
   
   * refactor: rename utils to wasm-interop
   
   * refactor: move KNOWN_STRATEGIES to Ygg
   
   * chore: make Clippy happy
   
   * test: fix tests
   
   * test: add e2e tests for custom strategies
   
   * chore: deltas in custom strategies
   
   * Update javascript-engine/src/index.ts
   
   * fix: imports

### New Features

 - <csr-id-fb10e5b302ec980aa90bbb4199c0bece39c82427/> expose enabled on list features in core and java
   * feat(java): expose enabled on list features in core and java

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 20 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#226](https://github.com/Unleash/yggdrasil/issues/226), [#230](https://github.com/Unleash/yggdrasil/issues/230)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#226](https://github.com/Unleash/yggdrasil/issues/226)**
    - Javascript engine ([`9096eea`](https://github.com/Unleash/yggdrasil/commit/9096eea4dbb24d71ae4876c976f85e5aa9472b04))
 * **[#230](https://github.com/Unleash/yggdrasil/issues/230)**
    - Expose enabled on list features in core and java ([`fb10e5b`](https://github.com/Unleash/yggdrasil/commit/fb10e5b302ec980aa90bbb4199c0bece39c82427))
</details>

## v0.17.2 (2025-02-19)

<csr-id-b5d787847fb8a3229854cc5e18cea51dca3f3cb6/>

### Chore

 - <csr-id-b5d787847fb8a3229854cc5e18cea51dca3f3cb6/> bump types to 0.15.9
   * chore: bump types to 0.15.9

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#222](https://github.com/Unleash/yggdrasil/issues/222)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#222](https://github.com/Unleash/yggdrasil/issues/222)**
    - Bump types to 0.15.9 ([`b5d7878`](https://github.com/Unleash/yggdrasil/commit/b5d787847fb8a3229854cc5e18cea51dca3f3cb6))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.17.2 ([`1db4699`](https://github.com/Unleash/yggdrasil/commit/1db46994c9908c85de1929f4766e691bd10dac6b))
</details>

## v0.17.1 (2025-02-19)

### Bug Fixes

 - <csr-id-feddb5d52a75f698ee58ce548ccd9cda0b04ad76/> metric bucket start time should be reset everytime it is sent
   * fix: metric bucket start time should be reset everytime it is sent
* chore: add test to cover that time is correctly advanced by metrics

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 13 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#219](https://github.com/Unleash/yggdrasil/issues/219)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#219](https://github.com/Unleash/yggdrasil/issues/219)**
    - Metric bucket start time should be reset everytime it is sent ([`feddb5d`](https://github.com/Unleash/yggdrasil/commit/feddb5d52a75f698ee58ce548ccd9cda0b04ad76))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.17.1 ([`9b1a14f`](https://github.com/Unleash/yggdrasil/commit/9b1a14f0b6369a64306fe78dd0a806a565b15a5d))
</details>

## v0.17.0 (2025-02-05)

### New Features

 - <csr-id-8143be838684cdffd59f050417578fc783d60a9c/> move state ingestion entry point
   * chore: move the switch that checks for delta into yggdrasil main take_state method so no one has to worry about it again
* chore: update client specs to include 19 and 20
* fix(ruby): patch custom strategies handling with delta events
* chore: bump client spec version for core and ruby to 5.2.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 5 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#213](https://github.com/Unleash/yggdrasil/issues/213)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#213](https://github.com/Unleash/yggdrasil/issues/213)**
    - Move state ingestion entry point ([`8143be8`](https://github.com/Unleash/yggdrasil/commit/8143be838684cdffd59f050417578fc783d60a9c))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.17.0 ([`f0ed1d5`](https://github.com/Unleash/yggdrasil/commit/f0ed1d594b5e73259c9fcb003e904caaaf225b3b))
</details>

## v0.16.1 (2025-01-30)

<csr-id-60a6f0bfc48c225ad8c2864019db5914bbb901e7/>
<csr-id-93953bca1a927c5819caeb2d56499133e81bd07d/>
<csr-id-a49d70d768bb4df7c1bf3161855455d5fe0429a6/>
<csr-id-a0632ef5ea76a60b44cc8105a29be4f335a9bdb2/>

### Chore

 - <csr-id-60a6f0bfc48c225ad8c2864019db5914bbb901e7/> update releasing template
 - <csr-id-93953bca1a927c5819caeb2d56499133e81bd07d/> bump types to 0.15.6
 - <csr-id-a49d70d768bb4df7c1bf3161855455d5fe0429a6/> bump types to 0.15.6
 - <csr-id-a0632ef5ea76a60b44cc8105a29be4f335a9bdb2/> optimizations to core layer
   * fix: repair benchmarks
   
   * chore: move fallback date into rule engine so that rules that don't need it don't incur the cost of creating a date

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#209](https://github.com/Unleash/yggdrasil/issues/209), [#210](https://github.com/Unleash/yggdrasil/issues/210)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#209](https://github.com/Unleash/yggdrasil/issues/209)**
    - Optimizations to core layer ([`a0632ef`](https://github.com/Unleash/yggdrasil/commit/a0632ef5ea76a60b44cc8105a29be4f335a9bdb2))
 * **[#210](https://github.com/Unleash/yggdrasil/issues/210)**
    - Bump types to 0.15.6 ([`93953bc`](https://github.com/Unleash/yggdrasil/commit/93953bca1a927c5819caeb2d56499133e81bd07d))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.16.1 ([`0215088`](https://github.com/Unleash/yggdrasil/commit/0215088778b52da701e31d9967c4f7502f7c60a7))
    - Update releasing template ([`60a6f0b`](https://github.com/Unleash/yggdrasil/commit/60a6f0bfc48c225ad8c2864019db5914bbb901e7))
    - Bump types to 0.15.6 ([`a49d70d`](https://github.com/Unleash/yggdrasil/commit/a49d70d768bb4df7c1bf3161855455d5fe0429a6))
</details>

## v0.16.0 (2025-01-29)

### New Features

 - <csr-id-d73a4fa0581b32e14d4efe3d763eb623e1ff3e65/> add hydration method to ruby to receive delta updates

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#206](https://github.com/Unleash/yggdrasil/issues/206)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#206](https://github.com/Unleash/yggdrasil/issues/206)**
    - Add hydration method to ruby to receive delta updates ([`d73a4fa`](https://github.com/Unleash/yggdrasil/commit/d73a4fa0581b32e14d4efe3d763eb623e1ff3e65))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.16.0 ([`9cb7625`](https://github.com/Unleash/yggdrasil/commit/9cb762563ce439f552a8d9ae804c4b4f1a09c25e))
</details>

## v0.15.0 (2025-01-29)

<csr-id-c685343d91c9f3ed64422d19915a74c4d52e823b/>

### Chore

 - <csr-id-c685343d91c9f3ed64422d19915a74c4d52e823b/> clean up some clippy warnings, remove an unnessary clone

### New Features

 - <csr-id-12350311c13a5f606bd8cde354b8a17f16586483/> implement new delta api format
   * feat: implement new delta api format
* feat: implement new delta api format
* feat: implement new delta api format
* feat: implement new delta api format
* feat: internal ygg feature toggle cache
* feat: rename to take_delta
* fix: fmt
* fix: rm comments
* fix: remove uneccesary setter
* fix: fmt

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 14 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#204](https://github.com/Unleash/yggdrasil/issues/204), [#205](https://github.com/Unleash/yggdrasil/issues/205)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#204](https://github.com/Unleash/yggdrasil/issues/204)**
    - Implement new delta api format ([`1235031`](https://github.com/Unleash/yggdrasil/commit/12350311c13a5f606bd8cde354b8a17f16586483))
 * **[#205](https://github.com/Unleash/yggdrasil/issues/205)**
    - Clean up some clippy warnings, remove an unnessary clone ([`c685343`](https://github.com/Unleash/yggdrasil/commit/c685343d91c9f3ed64422d19915a74c4d52e823b))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.15.0 ([`2da9cac`](https://github.com/Unleash/yggdrasil/commit/2da9cac60f7f82eeb43a9cc985aa6254d8822b73))
</details>

## v0.14.6 (2025-01-14)

<csr-id-f870bfdf0be8554152ddebe7eb173ed3e7286816/>

### Chore

 - <csr-id-f870bfdf0be8554152ddebe7eb173ed3e7286816/> bump unleash types to 0.15.4

### New Features

 - <csr-id-9ef5c467ece7948e56a1cf6c1b544614ec8a387e/> expose list_known_features and core version in java layer
   * feat(java): expose list_known_features
* refactor(java):separate resposibilities on java engine so static methods can be exposed and do memory management that should work on Java 18 and above (#198)
* refactor(java): rework java engine so that static methods can be exposed cleanly
* feat(java): expose core version in java engine layer (#199)
* fix(java): force native encoding to utf-8 so windows doesn't set it to something horrible (#200)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 6 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#197](https://github.com/Unleash/yggdrasil/issues/197), [#201](https://github.com/Unleash/yggdrasil/issues/201)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#197](https://github.com/Unleash/yggdrasil/issues/197)**
    - Expose list_known_features and core version in java layer ([`9ef5c46`](https://github.com/Unleash/yggdrasil/commit/9ef5c467ece7948e56a1cf6c1b544614ec8a387e))
 * **[#201](https://github.com/Unleash/yggdrasil/issues/201)**
    - Bump unleash types to 0.15.4 ([`f870bfd`](https://github.com/Unleash/yggdrasil/commit/f870bfdf0be8554152ddebe7eb173ed3e7286816))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.14.6 ([`fc48343`](https://github.com/Unleash/yggdrasil/commit/fc48343b87ce7471eb124b2277ce7c5f48227771))
</details>

## v0.14.5 (2025-01-08)

### New Features

 - <csr-id-49b3abbfc136ee44080eeaba50cb40fd5e3b4788/> delta processing implementation
   * feat: start work on delta processing
* First test running
* chore: update unleash-types for ffi and wasm
* Extended test coverage
* Extended test coverage
* Fix
* Fix
* Fix

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#188](https://github.com/Unleash/yggdrasil/issues/188)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#188](https://github.com/Unleash/yggdrasil/issues/188)**
    - Delta processing implementation ([`49b3abb`](https://github.com/Unleash/yggdrasil/commit/49b3abbfc136ee44080eeaba50cb40fd5e3b4788))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.14.5 ([`f699ee2`](https://github.com/Unleash/yggdrasil/commit/f699ee25230872980ccc1f1561440f699ecc8c9e))
</details>

## v0.14.4 (2025-01-08)

<csr-id-bc7c0c3be8653e812e295922685c77ea7b3e684f/>

### Chore

 - <csr-id-bc7c0c3be8653e812e295922685c77ea7b3e684f/> bump unleash-types version

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#194](https://github.com/Unleash/yggdrasil/issues/194)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#194](https://github.com/Unleash/yggdrasil/issues/194)**
    - Bump unleash-types version ([`bc7c0c3`](https://github.com/Unleash/yggdrasil/commit/bc7c0c3be8653e812e295922685c77ea7b3e684f))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.14.4 ([`b4a165d`](https://github.com/Unleash/yggdrasil/commit/b4a165d21445f934c176eed3687648e66a0dc603))
</details>

## v0.14.3 (2025-01-07)

<csr-id-21e0f7d2574e0032d65a13629bdcb666562891df/>

### Chore

 - <csr-id-21e0f7d2574e0032d65a13629bdcb666562891df/> update to most recent unleash-types

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#191](https://github.com/Unleash/yggdrasil/issues/191)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#191](https://github.com/Unleash/yggdrasil/issues/191)**
    - Update to most recent unleash-types ([`21e0f7d`](https://github.com/Unleash/yggdrasil/commit/21e0f7d2574e0032d65a13629bdcb666562891df))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.14.3 ([`7457210`](https://github.com/Unleash/yggdrasil/commit/745721034f8744503efe21cf66f38ab350948a40))
</details>

## v0.14.2 (2025-01-07)

### Bug Fixes

 - <csr-id-0aae4be995b8a49df0f75fe73a788cf546c93a8d/> support for utf-8 chars on win
   * fix: support for utf-8 chars on win
* chore: bump client spec

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 40 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#189](https://github.com/Unleash/yggdrasil/issues/189)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#189](https://github.com/Unleash/yggdrasil/issues/189)**
    - Support for utf-8 chars on win ([`0aae4be`](https://github.com/Unleash/yggdrasil/commit/0aae4be995b8a49df0f75fe73a788cf546c93a8d))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.14.2 ([`040c250`](https://github.com/Unleash/yggdrasil/commit/040c2501663660844c16364d578c0d991fbc5db5))
</details>

## v0.14.1 (2024-11-27)

### Bug Fixes

 - <csr-id-ce88e4723dce2726fa92c0d4578ff0409cceae3e/> make those annoying hostname tests run in serial so they don't flake

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 19 calendar days.
 - 29 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#166](https://github.com/Unleash/yggdrasil/issues/166)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#166](https://github.com/Unleash/yggdrasil/issues/166)**
    - Make those annoying hostname tests run in serial so they don't flake ([`ce88e47`](https://github.com/Unleash/yggdrasil/commit/ce88e4723dce2726fa92c0d4578ff0409cceae3e))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.14.1 ([`6ddce2b`](https://github.com/Unleash/yggdrasil/commit/6ddce2b53457b60740a5a1e3803b52f1acb2082a))
</details>

## v0.14.0 (2024-10-29)

<csr-id-10369a0bd4b6139709aeb144ea23a6a51fa56a54/>

### Chore

 - <csr-id-10369a0bd4b6139709aeb144ea23a6a51fa56a54/> add autoformatting config

### New Features

 - <csr-id-443f662a614ef3c207d8c4974b374db5ae93b956/> list known features

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 6 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#156](https://github.com/Unleash/yggdrasil/issues/156), [#157](https://github.com/Unleash/yggdrasil/issues/157)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#156](https://github.com/Unleash/yggdrasil/issues/156)**
    - Add autoformatting config ([`10369a0`](https://github.com/Unleash/yggdrasil/commit/10369a0bd4b6139709aeb144ea23a6a51fa56a54))
 * **[#157](https://github.com/Unleash/yggdrasil/issues/157)**
    - List known features ([`443f662`](https://github.com/Unleash/yggdrasil/commit/443f662a614ef3c207d8c4974b374db5ae93b956))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.14.0 ([`30cd26e`](https://github.com/Unleash/yggdrasil/commit/30cd26e65fa756893154839045ea241ab1442f53))
</details>

## v0.13.3 (2024-10-22)

<csr-id-f8310a76fc2f23eb39127e88bbcef7bb49ff528c/>

### Chore

 - <csr-id-f8310a76fc2f23eb39127e88bbcef7bb49ff528c/> ARM ruby build

### Bug Fixes

 - <csr-id-fc9097467a2985d37559c7850b5f78b37a5b51fc/> move wasm engine to handle featureEnabled not feature_enabled

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 28 calendar days.
 - 32 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#145](https://github.com/Unleash/yggdrasil/issues/145), [#149](https://github.com/Unleash/yggdrasil/issues/149)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#145](https://github.com/Unleash/yggdrasil/issues/145)**
    - ARM ruby build ([`f8310a7`](https://github.com/Unleash/yggdrasil/commit/f8310a76fc2f23eb39127e88bbcef7bb49ff528c))
 * **[#149](https://github.com/Unleash/yggdrasil/issues/149)**
    - Move wasm engine to handle featureEnabled not feature_enabled ([`fc90974`](https://github.com/Unleash/yggdrasil/commit/fc9097467a2985d37559c7850b5f78b37a5b51fc))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.13.3 ([`c2f0cb7`](https://github.com/Unleash/yggdrasil/commit/c2f0cb758eb00d26d307aa7c76052f3b5529f6c6))
</details>

## v0.13.2 (2024-09-20)

<csr-id-33cd400eb57f52386ba9d583a68b0401a14b47a8/>

### Chore

 - <csr-id-33cd400eb57f52386ba9d583a68b0401a14b47a8/> make wasm build again
   * chore: make wasm build again
   
   * chore: make hostname a feature instead
   
   * fix: put tests behind feature, include env when needed
   
   * refactor: organize imports slightly differently
   
   * chore: test all features

### Bug Fixes

 - <csr-id-5c8ba06563dc48a13188fe1fabb4794e5959ef9c/> makes random distribution better distributed

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 13 calendar days.
 - 14 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#141](https://github.com/Unleash/yggdrasil/issues/141), [#144](https://github.com/Unleash/yggdrasil/issues/144)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#141](https://github.com/Unleash/yggdrasil/issues/141)**
    - Make wasm build again ([`33cd400`](https://github.com/Unleash/yggdrasil/commit/33cd400eb57f52386ba9d583a68b0401a14b47a8))
 * **[#144](https://github.com/Unleash/yggdrasil/issues/144)**
    - Makes random distribution better distributed ([`5c8ba06`](https://github.com/Unleash/yggdrasil/commit/5c8ba06563dc48a13188fe1fabb4794e5959ef9c))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.13.2 ([`0286e61`](https://github.com/Unleash/yggdrasil/commit/0286e612a087cac48f62df056afa1b579ccc7d77))
</details>

## v0.13.1 (2024-09-06)

<csr-id-645f6062535ff1ee966ed544639864d4872a1e55/>

### Chore

 - <csr-id-645f6062535ff1ee966ed544639864d4872a1e55/> add changelog

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 48 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#139](https://github.com/Unleash/yggdrasil/issues/139)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#139](https://github.com/Unleash/yggdrasil/issues/139)**
    - Fix/inversion always supported ([`9bd0aac`](https://github.com/Unleash/yggdrasil/commit/9bd0aace1b304b6c9660bb52f63abd2fa0755a62))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.13.1 ([`f8de91a`](https://github.com/Unleash/yggdrasil/commit/f8de91a2516fb70616e23c68357c75fad7a6ac2d))
    - Add changelog ([`645f606`](https://github.com/Unleash/yggdrasil/commit/645f6062535ff1ee966ed544639864d4872a1e55))
    - Adjusting changelogs prior to release of unleash-yggdrasil v0.13.1 ([`2130699`](https://github.com/Unleash/yggdrasil/commit/2130699da93d945f35db5d27ce742f3abdc4272f))
    - Merge branch 'main' of github.com:Unleash/yggdrasil ([`abc78e7`](https://github.com/Unleash/yggdrasil/commit/abc78e7ba019dae94536d200e5ed7f76e2b43847))
</details>

## v0.13.0 (2024-07-19)

<csr-id-4743bd064343e4971dbcb8098edf73cc4219e962/>
<csr-id-c1b10122322044b12e62c7bf873982c2c46f3c3f/>

### Chore

 - <csr-id-4743bd064343e4971dbcb8098edf73cc4219e962/> update unleash-types and chrono
 - <csr-id-c1b10122322044b12e62c7bf873982c2c46f3c3f/> apply some lints and fixes

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 23 calendar days.
 - 84 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#130](https://github.com/Unleash/yggdrasil/issues/130)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#130](https://github.com/Unleash/yggdrasil/issues/130)**
    - Apply some lints and fixes ([`c1b1012`](https://github.com/Unleash/yggdrasil/commit/c1b10122322044b12e62c7bf873982c2c46f3c3f))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.13.0, safety bump 2 crates ([`b6c0a25`](https://github.com/Unleash/yggdrasil/commit/b6c0a25793de0c6d2f3ef0cfe2f6c7c753320b96))
    - Update unleash-types and chrono ([`4743bd0`](https://github.com/Unleash/yggdrasil/commit/4743bd064343e4971dbcb8098edf73cc4219e962))
</details>

## v0.12.0 (2024-04-25)

<csr-id-1912fdaddb4f4adcfbd3cf2e5b3b11fc50366794/>

### Chore

 - <csr-id-1912fdaddb4f4adcfbd3cf2e5b3b11fc50366794/> bump unleash types to 0.12

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 56 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#124](https://github.com/Unleash/yggdrasil/issues/124)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#124](https://github.com/Unleash/yggdrasil/issues/124)**
    - Bump unleash types to 0.12 ([`1912fda`](https://github.com/Unleash/yggdrasil/commit/1912fdaddb4f4adcfbd3cf2e5b3b11fc50366794))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.12.0 ([`5c82cb1`](https://github.com/Unleash/yggdrasil/commit/5c82cb1f53c7944237c4a87403d0e727c872ec70))
</details>

## v0.11.0 (2024-02-29)

<csr-id-39b586ff9b4b44d7f6660c5cedbaf7003e3fc36a/>

### Chore

 - <csr-id-39b586ff9b4b44d7f6660c5cedbaf7003e3fc36a/> allow take_state to bubble warnings to caller

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#123](https://github.com/Unleash/yggdrasil/issues/123)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#123](https://github.com/Unleash/yggdrasil/issues/123)**
    - Allow take_state to bubble warnings to caller ([`39b586f`](https://github.com/Unleash/yggdrasil/commit/39b586ff9b4b44d7f6660c5cedbaf7003e3fc36a))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.11.0 ([`ddb8216`](https://github.com/Unleash/yggdrasil/commit/ddb82163acf9246214478202f1e20f4078609e8a))
</details>

## v0.10.0 (2024-02-29)

<csr-id-3a72666241cb30d45d3a52cbca723f996570713b/>
<csr-id-b8cecbcab3a39e353a259fd0a7517b9301857f57/>

### Chore

 - <csr-id-3a72666241cb30d45d3a52cbca723f996570713b/> remove orig file

### Bug Fixes

 - <csr-id-d5023d13858e23c2ffababe70e398a340ba00a6b/> toggle compile failures now bubble
 - <csr-id-cc58f306862bb1105732ffc73c653e240f333a46/> one toggle failing to compile will no longer affect other toggles

### Refactor

 - <csr-id-b8cecbcab3a39e353a259fd0a7517b9301857f57/> better handling for unwrapping grammar nodes

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 36 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#120](https://github.com/Unleash/yggdrasil/issues/120), [#121](https://github.com/Unleash/yggdrasil/issues/121), [#122](https://github.com/Unleash/yggdrasil/issues/122)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#120](https://github.com/Unleash/yggdrasil/issues/120)**
    - One toggle failing to compile will no longer affect other toggles ([`cc58f30`](https://github.com/Unleash/yggdrasil/commit/cc58f306862bb1105732ffc73c653e240f333a46))
 * **[#121](https://github.com/Unleash/yggdrasil/issues/121)**
    - Better handling for unwrapping grammar nodes ([`b8cecbc`](https://github.com/Unleash/yggdrasil/commit/b8cecbcab3a39e353a259fd0a7517b9301857f57))
 * **[#122](https://github.com/Unleash/yggdrasil/issues/122)**
    - Toggle compile failures now bubble ([`d5023d1`](https://github.com/Unleash/yggdrasil/commit/d5023d13858e23c2ffababe70e398a340ba00a6b))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.10.0 ([`e6adf79`](https://github.com/Unleash/yggdrasil/commit/e6adf79f80abf760ba3f07eaced8601eb1cee655))
    - Release unleash-yggdrasil v0.10.0 ([`e528c7a`](https://github.com/Unleash/yggdrasil/commit/e528c7acaf9c06575212541dfcb2b8e1c71b7bb6))
    - Remove orig file ([`3a72666`](https://github.com/Unleash/yggdrasil/commit/3a72666241cb30d45d3a52cbca723f996570713b))
</details>

## v0.9.0 (2024-01-23)

<csr-id-799674dcf4f2937451f815525e60017d7e8ae830/>
<csr-id-1611a117751d847763acc4f57592c627c39d4523/>

### Chore

 - <csr-id-799674dcf4f2937451f815525e60017d7e8ae830/> update types lib
 - <csr-id-1611a117751d847763acc4f57592c627c39d4523/> bump unleash-types and chronos

### New Features

 - <csr-id-f4f6ab8f1ca9e03c7c46c8e6b132aac1d207f2e9/> add handling for remote address strategy for subnets and IPv6
 - <csr-id-96135082118aed41db212df32ebcbcd901d0fb82/> implement hostname strategy in core
 - <csr-id-5225301a1c8c5922bb3314ddf5ed513da39302c3/> add feature enabled property to variant checks
 - <csr-id-054cac7ccbf1b5127de3b2f5f761f6464ffd725a/> Bubble up errors on take state
   * Add test for parse error
* Propagate errors in parse date

### Bug Fixes

 - <csr-id-d7158ce28aed33583a3316af127e3631415d4f65/> hostname strategy is now generated correctly and ignores case
 - <csr-id-9e881fc9124da149033aceb154cd9b627ce65ed6/> strategies that are broken will now default to false

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 62 calendar days.
 - 71 days passed between releases.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 9 unique issues were worked on: [#101](https://github.com/Unleash/yggdrasil/issues/101), [#102](https://github.com/Unleash/yggdrasil/issues/102), [#103](https://github.com/Unleash/yggdrasil/issues/103), [#109](https://github.com/Unleash/yggdrasil/issues/109), [#110](https://github.com/Unleash/yggdrasil/issues/110), [#114](https://github.com/Unleash/yggdrasil/issues/114), [#118](https://github.com/Unleash/yggdrasil/issues/118), [#97](https://github.com/Unleash/yggdrasil/issues/97), [#98](https://github.com/Unleash/yggdrasil/issues/98)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#101](https://github.com/Unleash/yggdrasil/issues/101)**
    - Implement hostname strategy in core ([`9613508`](https://github.com/Unleash/yggdrasil/commit/96135082118aed41db212df32ebcbcd901d0fb82))
 * **[#102](https://github.com/Unleash/yggdrasil/issues/102)**
    - Strategies that are broken will now default to false ([`9e881fc`](https://github.com/Unleash/yggdrasil/commit/9e881fc9124da149033aceb154cd9b627ce65ed6))
 * **[#103](https://github.com/Unleash/yggdrasil/issues/103)**
    - Hostname strategy is now generated correctly and ignores case ([`d7158ce`](https://github.com/Unleash/yggdrasil/commit/d7158ce28aed33583a3316af127e3631415d4f65))
 * **[#109](https://github.com/Unleash/yggdrasil/issues/109)**
    - Bump unleash-types and chronos ([`1611a11`](https://github.com/Unleash/yggdrasil/commit/1611a117751d847763acc4f57592c627c39d4523))
 * **[#110](https://github.com/Unleash/yggdrasil/issues/110)**
    - Add handling for remote address strategy for subnets and IPv6 ([`f4f6ab8`](https://github.com/Unleash/yggdrasil/commit/f4f6ab8f1ca9e03c7c46c8e6b132aac1d207f2e9))
 * **[#114](https://github.com/Unleash/yggdrasil/issues/114)**
    - Fix(core)/missing strategy variants no longer impacts other strategies ([`1f58217`](https://github.com/Unleash/yggdrasil/commit/1f58217f8da9536abcaa4ddd25003541964ab1ad))
 * **[#118](https://github.com/Unleash/yggdrasil/issues/118)**
    - Update types lib ([`799674d`](https://github.com/Unleash/yggdrasil/commit/799674dcf4f2937451f815525e60017d7e8ae830))
 * **[#97](https://github.com/Unleash/yggdrasil/issues/97)**
    - Bubble up errors on take state ([`054cac7`](https://github.com/Unleash/yggdrasil/commit/054cac7ccbf1b5127de3b2f5f761f6464ffd725a))
 * **[#98](https://github.com/Unleash/yggdrasil/issues/98)**
    - Add feature enabled property to variant checks ([`5225301`](https://github.com/Unleash/yggdrasil/commit/5225301a1c8c5922bb3314ddf5ed513da39302c3))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.9.0 ([`fd92260`](https://github.com/Unleash/yggdrasil/commit/fd92260da07fb3cda2a1fa526ad84bc70d9a1283))
</details>

## v0.8.0 (2023-11-13)

<csr-id-fbfbe329dfbf435e4a010769bd118e6a9e347325/>
<csr-id-adb57f62ea8fd3db99107fdb22c562371c7032b0/>

### Chore

 - <csr-id-fbfbe329dfbf435e4a010769bd118e6a9e347325/> apply cargo fmt

### New Features

<csr-id-77537e0b45f197fa8ed43e061e1107dabb5c2a57/>

 - <csr-id-8c5330b1b6935ecd695058b9c7aa79d3f7766122/> add shouldemitimpressionevents to yggdrasil and .NET wrapper
   * feat: add shouldemitimpressionevents to yggdrasil and .NET wrapper
* feat: added some impression events tests in dotnet engine

### Bug Fixes

 - <csr-id-6919241817638a7005974b0b85753e2464faa0fe/> make free floating quotes in string operators work correctly

### Other

 - <csr-id-adb57f62ea8fd3db99107fdb22c562371c7032b0/> fix normalized hash result to start from 1 instead of 0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 12 calendar days.
 - 13 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 5 unique issues were worked on: [#56](https://github.com/Unleash/yggdrasil/issues/56), [#63](https://github.com/Unleash/yggdrasil/issues/63), [#65](https://github.com/Unleash/yggdrasil/issues/65), [#75](https://github.com/Unleash/yggdrasil/issues/75), [#80](https://github.com/Unleash/yggdrasil/issues/80)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#56](https://github.com/Unleash/yggdrasil/issues/56)**
    - Custom strategies in ruby and support in the core engine ([`77537e0`](https://github.com/Unleash/yggdrasil/commit/77537e0b45f197fa8ed43e061e1107dabb5c2a57))
 * **[#63](https://github.com/Unleash/yggdrasil/issues/63)**
    - Apply cargo fmt ([`fbfbe32`](https://github.com/Unleash/yggdrasil/commit/fbfbe329dfbf435e4a010769bd118e6a9e347325))
 * **[#65](https://github.com/Unleash/yggdrasil/issues/65)**
    - Fix normalized hash result to start from 1 instead of 0 ([`adb57f6`](https://github.com/Unleash/yggdrasil/commit/adb57f62ea8fd3db99107fdb22c562371c7032b0))
 * **[#75](https://github.com/Unleash/yggdrasil/issues/75)**
    - Add shouldemitimpressionevents to yggdrasil and .NET wrapper ([`8c5330b`](https://github.com/Unleash/yggdrasil/commit/8c5330b1b6935ecd695058b9c7aa79d3f7766122))
 * **[#80](https://github.com/Unleash/yggdrasil/issues/80)**
    - Make free floating quotes in string operators work correctly ([`6919241`](https://github.com/Unleash/yggdrasil/commit/6919241817638a7005974b0b85753e2464faa0fe))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.8.0, safety bump 2 crates ([`d11c481`](https://github.com/Unleash/yggdrasil/commit/d11c4819117491ad882d516807f54ae5608e8277))
    - Release unleash-yggdrasil v0.7.0 ([`d9204d6`](https://github.com/Unleash/yggdrasil/commit/d9204d6e4301f64a038f2bed447bf3db08c0f8a8))
</details>

## v0.7.0 (2023-10-30)

<csr-id-adb57f62ea8fd3db99107fdb22c562371c7032b0/>
<csr-id-fbfbe329dfbf435e4a010769bd118e6a9e347325/>

### Other

 - <csr-id-adb57f62ea8fd3db99107fdb22c562371c7032b0/> fix normalized hash result to start from 1 instead of 0

### Bug Fixes

 - <csr-id-6919241817638a7005974b0b85753e2464faa0fe/> make free floating quotes in string operators work correctly

### Chore

 - <csr-id-fbfbe329dfbf435e4a010769bd118e6a9e347325/> apply cargo fmt

### New Features

<csr-id-77537e0b45f197fa8ed43e061e1107dabb5c2a57/>

 - <csr-id-df0d85445a91d987219fc60d26f2c6c097c0d878/> new hashing implemented for variants
 - <csr-id-8c5330b1b6935ecd695058b9c7aa79d3f7766122/> add shouldemitimpressionevents to yggdrasil and .NET wrapper
   * feat: add shouldemitimpressionevents to yggdrasil and .NET wrapper
* feat: added some impression events tests in dotnet engine

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 17 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#61](https://github.com/Unleash/yggdrasil/issues/61)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#61](https://github.com/Unleash/yggdrasil/issues/61)**
    - New hashing implemented for variants ([`df0d854`](https://github.com/Unleash/yggdrasil/commit/df0d85445a91d987219fc60d26f2c6c097c0d878))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.7.0 ([`b57e44c`](https://github.com/Unleash/yggdrasil/commit/b57e44cbcd79973896c6facd50f91866c1641a4a))
    - Release unleash-yggdrasil v0.7.0 ([`38b87b5`](https://github.com/Unleash/yggdrasil/commit/38b87b52d980fe623d683d4d9bd792bbf34eeff2))
</details>

## v0.6.1 (2023-10-13)

### New Features

 - <csr-id-d89c1837e1ef965eaea49e2b54c5a71bfa475877/> add dependent flags
   ### Description
   
   This PR adds dependent flags to Yggdrasil, complying with the updated client specs.
   
   Regarding these extra points not covered by the spec:
   
   metrics are not called on dependent features :: this is covered and tested
   impression events are called on dependent features for easier debugging :: yggdrasil doesn't touch impression events
   warning events for missing dependencies are reported once :: yggdrasil doesn't deal with warnings
   
   @kwasniew , there is a line in resolveVariant in the Node SDK that checks for parent dependencies being satisfied. What exactly does it do? I tried doing something similar here, but removed it because the tests pass either way. Is this an edge case not covered by the tests, or is it just that the implementations differ here?
   
   
   ### (Important) Commits
   
   * feat(#2255): update client specs
* Wip: impl 1 (without updating unleash-types)
* feat(#2255): add test to check that metrics aren't recorded
* feat(#2255): fix assumption in test
* feat(#2255): avoid counting metrics for parents
* feat(#2255): make deps always a vec
* feat(#2255): use unleash-types type for dependency
* feat(#2255): bump unleash-types
* feat(#2255): add test for metrics if parent flag has variants
* Get variant without counting
* feat(#2255): add test to ensure get_variant works

### Bug Fixes

 - <csr-id-8bf7a697411cd9e4b029151253740d67104faa88/> allows current time to be assumed when calculating the context

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#49](https://github.com/Unleash/yggdrasil/issues/49), [#51](https://github.com/Unleash/yggdrasil/issues/51)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#49](https://github.com/Unleash/yggdrasil/issues/49)**
    - Add dependent flags ([`d89c183`](https://github.com/Unleash/yggdrasil/commit/d89c1837e1ef965eaea49e2b54c5a71bfa475877))
 * **[#51](https://github.com/Unleash/yggdrasil/issues/51)**
    - Allows current time to be assumed when calculating the context ([`8bf7a69`](https://github.com/Unleash/yggdrasil/commit/8bf7a697411cd9e4b029151253740d67104faa88))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.6.1 ([`7cc8312`](https://github.com/Unleash/yggdrasil/commit/7cc831213005a619452121074a997d4a23a6f7b4))
    - Release unleash-yggdrasil v0.6.1 ([`32c21f6`](https://github.com/Unleash/yggdrasil/commit/32c21f6155deb4c1b7331f11c18e9a7a232f9489))
</details>

## v0.6.0 (2023-10-10)

<csr-id-e08f62b77a8eb20121989fecf2b0161f9dc34215/>

### New Features

 - <csr-id-c878e9535018ecc10f99419528d1f6b8d2e9d6c5/> make ruby useful
   This introduces a useful set of code for Ruby. This is very much a spike, there are still some potential leaks in this code and there's no build system to actually emit the artefacts we actually need to consume this in another library. But the logic appears to be correct overall and this is a useful spike to building the other ffi consumers

### Bug Fixes

 - <csr-id-137e4c8364f62262d7968c21e012e06083b6bd50/> non strategy variants will work with empty strategy variants

### Refactor

 - <csr-id-e08f62b77a8eb20121989fecf2b0161f9dc34215/> better public api for ffi consumers
   * refactor: redesign metrics to better capture missing toggles/variants and be easier to pass
   
   * refactor: slightly better internal handling for counting and take state

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 56 calendar days.
 - 84 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#42](https://github.com/Unleash/yggdrasil/issues/42), [#43](https://github.com/Unleash/yggdrasil/issues/43), [#48](https://github.com/Unleash/yggdrasil/issues/48)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#42](https://github.com/Unleash/yggdrasil/issues/42)**
    - Better public api for ffi consumers ([`e08f62b`](https://github.com/Unleash/yggdrasil/commit/e08f62b77a8eb20121989fecf2b0161f9dc34215))
 * **[#43](https://github.com/Unleash/yggdrasil/issues/43)**
    - Make ruby useful ([`c878e95`](https://github.com/Unleash/yggdrasil/commit/c878e9535018ecc10f99419528d1f6b8d2e9d6c5))
 * **[#48](https://github.com/Unleash/yggdrasil/issues/48)**
    - Non strategy variants will work with empty strategy variants ([`137e4c8`](https://github.com/Unleash/yggdrasil/commit/137e4c8364f62262d7968c21e012e06083b6bd50))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.6.0 ([`26f86b6`](https://github.com/Unleash/yggdrasil/commit/26f86b6023fa8be52a20158aade78646550c00cf))
</details>

## v0.5.9 (2023-07-18)

<csr-id-7707e875027b6e100205ae5e3a2fd416f4af17ea/>

### Chore

 - <csr-id-7707e875027b6e100205ae5e3a2fd416f4af17ea/> updated to new chrono and unleash-types

### New Features

 - <csr-id-135639fca552048221a626ff5e3beef1c35d7825/> add support for strategy variants

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 20 calendar days.
 - 25 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#40](https://github.com/Unleash/yggdrasil/issues/40), [#41](https://github.com/Unleash/yggdrasil/issues/41)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#40](https://github.com/Unleash/yggdrasil/issues/40)**
    - Updated to new chrono and unleash-types ([`7707e87`](https://github.com/Unleash/yggdrasil/commit/7707e875027b6e100205ae5e3a2fd416f4af17ea))
 * **[#41](https://github.com/Unleash/yggdrasil/issues/41)**
    - Add support for strategy variants ([`135639f`](https://github.com/Unleash/yggdrasil/commit/135639fca552048221a626ff5e3beef1c35d7825))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.9 ([`851f488`](https://github.com/Unleash/yggdrasil/commit/851f488a2e4ec4c5217ffaa221cbc62aef5f6d2f))
</details>

## v0.5.8 (2023-06-23)

### New Features

 - <csr-id-a14f97229ecf31eeef003d74b29b09e319f2d394/> add single toggle resolve method
 - <csr-id-b36de19e5c13add657273eff3c42ba204a31e860/> basic FFI language bindings

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 18 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#39](https://github.com/Unleash/yggdrasil/issues/39)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#39](https://github.com/Unleash/yggdrasil/issues/39)**
    - Add single toggle resolve method ([`a14f972`](https://github.com/Unleash/yggdrasil/commit/a14f97229ecf31eeef003d74b29b09e319f2d394))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.8 ([`64828b7`](https://github.com/Unleash/yggdrasil/commit/64828b70dd0a26336ba3113e585dab077dcb15e1))
    - Basic FFI language bindings ([`b36de19`](https://github.com/Unleash/yggdrasil/commit/b36de19e5c13add657273eff3c42ba204a31e860))
</details>

## v0.5.7 (2023-06-05)

### Bug Fixes

 - <csr-id-280670e46cf654838d67817ade2b8963aaeb6198/> support for arbitrary strings
   * fix: allow grammar to parse more or less any string that doesn't contain the character sequence "

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 18 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#37](https://github.com/Unleash/yggdrasil/issues/37)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#37](https://github.com/Unleash/yggdrasil/issues/37)**
    - Support for arbitrary strings ([`280670e`](https://github.com/Unleash/yggdrasil/commit/280670e46cf654838d67817ade2b8963aaeb6198))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.7 ([`fb00e58`](https://github.com/Unleash/yggdrasil/commit/fb00e58fbd151cbe149b0a0c6b6fff466fed0781))
</details>

## v0.5.6 (2023-05-17)

### Bug Fixes

 - <csr-id-afb1cf8512def7cfdbea4dde9e667fe4424e1bdf/> redesigns the way stickiness is handled in the grammar to support default and random

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 15 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#36](https://github.com/Unleash/yggdrasil/issues/36)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#36](https://github.com/Unleash/yggdrasil/issues/36)**
    - Redesigns the way stickiness is handled in the grammar to support default and random ([`afb1cf8`](https://github.com/Unleash/yggdrasil/commit/afb1cf8512def7cfdbea4dde9e667fe4424e1bdf))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.6 ([`796dccb`](https://github.com/Unleash/yggdrasil/commit/796dccbd5e1d28e47d293cab1c6a07d0c8b2cc16))
</details>

## v0.5.5 (2023-05-02)

### Bug Fixes

 - <csr-id-acf5cc1007262675b0f5a03589ad0a62fd2c4fa6/> fix variant stickiness

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 13 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#34](https://github.com/Unleash/yggdrasil/issues/34)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#34](https://github.com/Unleash/yggdrasil/issues/34)**
    - Fix variant stickiness ([`acf5cc1`](https://github.com/Unleash/yggdrasil/commit/acf5cc1007262675b0f5a03589ad0a62fd2c4fa6))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.5 ([`c445d78`](https://github.com/Unleash/yggdrasil/commit/c445d78f98b559778649cf89bb95cbc59d027ff3))
</details>

## v0.5.4 (2023-04-19)

<csr-id-0c1edd38338e9ad0659ed80218aabd8235c36899/>

### Other

 - <csr-id-0c1edd38338e9ad0659ed80218aabd8235c36899/> Bump unleash types to 0.10.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 46 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#33](https://github.com/Unleash/yggdrasil/issues/33)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#33](https://github.com/Unleash/yggdrasil/issues/33)**
    - Bump unleash types to 0.10.0 ([`0c1edd3`](https://github.com/Unleash/yggdrasil/commit/0c1edd38338e9ad0659ed80218aabd8235c36899))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.4 ([`4da3483`](https://github.com/Unleash/yggdrasil/commit/4da34832edb373cda59ecd80f961ed2fc6d80918))
</details>

## v0.5.3 (2023-03-03)

### New Features

 - <csr-id-6f8bd369287000d248608a3ac4b2d062326c8cab/> add support for variants overrides on arbitrary context fields

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#31](https://github.com/Unleash/yggdrasil/issues/31)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#31](https://github.com/Unleash/yggdrasil/issues/31)**
    - Add support for variants overrides on arbitrary context fields ([`6f8bd36`](https://github.com/Unleash/yggdrasil/commit/6f8bd369287000d248608a3ac4b2d062326c8cab))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.3 ([`b58f062`](https://github.com/Unleash/yggdrasil/commit/b58f06258e4f8540a93afe9dd3b5f264d84adc77))
</details>

## v0.5.2 (2023-03-02)

<csr-id-66bf3d804482c0def165f2ebe282ac72c8917c57/>
<csr-id-1e4681329b1ca5436966c22c0e892b7a024496b9/>
<csr-id-d996bcb37c9e0571ccc32dd130408b9b655cc406/>
<csr-id-5ec32accf621858f74fc454d4e99616eb96b7a38/>
<csr-id-ee13f4ed2a6a1a13f70533fe568400bd2411742e/>
<csr-id-36e55c6ce33f0710e7f41f72dab86e01ddff6707/>
<csr-id-083f83994e44840647e6615164ebd17d781fd236/>
<csr-id-e1619d527175c197ff36bdaf9e57f6bbef0e17bf/>
<csr-id-6e73b31448a39ecf1b194fb24039f7b3d6bf533c/>
<csr-id-01d5aefc52b07c2616116518b5de68d6ac36788c/>
<csr-id-6e9688a251cc7a5ce77eb6ceef41a9038b9ff0a4/>
<csr-id-45787e38e529427bd8e351a3c69f38f84e98657f/>
<csr-id-9b015f543b914e028b54717b2c79b4c04a8f3d8c/>

### Chore

 - <csr-id-66bf3d804482c0def165f2ebe282ac72c8917c57/> bump version to 0.5.1
 - <csr-id-1e4681329b1ca5436966c22c0e892b7a024496b9/> bump unleash types to 0.9.0
 - <csr-id-d996bcb37c9e0571ccc32dd130408b9b655cc406/> bump version to 0.4.5
 - <csr-id-5ec32accf621858f74fc454d4e99616eb96b7a38/> bump version to 0.4.4 and unleash types to 0.8.1
 - <csr-id-ee13f4ed2a6a1a13f70533fe568400bd2411742e/> bump types to 0.8.0 and upgrade version to 0.4.3
 - <csr-id-36e55c6ce33f0710e7f41f72dab86e01ddff6707/> add basic benchmarks
 - <csr-id-083f83994e44840647e6615164ebd17d781fd236/> bump version to 0.4.1
 - <csr-id-e1619d527175c197ff36bdaf9e57f6bbef0e17bf/> bump version to 0.4.0
 - <csr-id-6e73b31448a39ecf1b194fb24039f7b3d6bf533c/> remove context object from this project
 - <csr-id-01d5aefc52b07c2616116518b5de68d6ac36788c/> bump version to 0.3.0
 - <csr-id-6e9688a251cc7a5ce77eb6ceef41a9038b9ff0a4/> update unleash-types to 0.4.1 to deal with variant weight type response from unleash
 - <csr-id-45787e38e529427bd8e351a3c69f38f84e98657f/> bump version to 0.2.0

### Chore

 - <csr-id-9b015f543b914e028b54717b2c79b4c04a8f3d8c/> include changelog to prep for smart release

### New Features

<csr-id-0b5437ff5e6417bf8c2886ed97aad77c3b5b698a/>
<csr-id-d3f94be8ea473a8f3d465daa24bddbee487c8d45/>
<csr-id-2b6f88924d8c7a6dd592ac816fd0a86fb223db37/>
<csr-id-941bb90e5c97cd14e843f55ed946005a2de811ea/>

 - <csr-id-7f402c238163d83cd8329dc8c4cbd005c4182a9e/> expose project on resolved toggle
 - <csr-id-1617775d48207454bda7bc373de2bf0da7dd04cc/> add resolve all features method
   * chore: move to version 0.5.0
* feat: adds a new function on the engine to resolve all features states currently loaded
* fix: pass the toggle name through to the rule engine so that group id can be properly calculated
* fix: add fallback handling for get_variant in cases where custom stickiness is defined but no context property is present for it

### Bug Fixes

 - <csr-id-58d1cae7fbfa78f44adccf7c098011fc047f5c9c/> handle cases where get_variant is called and variants is Some but empty
 - <csr-id-0a6a19b9be93c85b095f9f1bc4f1eb8b5cfd522c/> upgrade user id strategy to string types rather than numerics
 - <csr-id-2384c165aca366a6e7fdf4d7f688fd9949931471/> remove reqwest, we don't use it

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 30 commits contributed to the release over the course of 44 calendar days.
 - 22 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 25 unique issues were worked on: [#11](https://github.com/Unleash/yggdrasil/issues/11), [#12](https://github.com/Unleash/yggdrasil/issues/12), [#13](https://github.com/Unleash/yggdrasil/issues/13), [#14](https://github.com/Unleash/yggdrasil/issues/14), [#15](https://github.com/Unleash/yggdrasil/issues/15), [#16](https://github.com/Unleash/yggdrasil/issues/16), [#17](https://github.com/Unleash/yggdrasil/issues/17), [#18](https://github.com/Unleash/yggdrasil/issues/18), [#19](https://github.com/Unleash/yggdrasil/issues/19), [#2](https://github.com/Unleash/yggdrasil/issues/2), [#20](https://github.com/Unleash/yggdrasil/issues/20), [#21](https://github.com/Unleash/yggdrasil/issues/21), [#23](https://github.com/Unleash/yggdrasil/issues/23), [#24](https://github.com/Unleash/yggdrasil/issues/24), [#25](https://github.com/Unleash/yggdrasil/issues/25), [#27](https://github.com/Unleash/yggdrasil/issues/27), [#28](https://github.com/Unleash/yggdrasil/issues/28), [#29](https://github.com/Unleash/yggdrasil/issues/29), [#30](https://github.com/Unleash/yggdrasil/issues/30), [#4](https://github.com/Unleash/yggdrasil/issues/4), [#5](https://github.com/Unleash/yggdrasil/issues/5), [#6](https://github.com/Unleash/yggdrasil/issues/6), [#7](https://github.com/Unleash/yggdrasil/issues/7), [#8](https://github.com/Unleash/yggdrasil/issues/8), [#9](https://github.com/Unleash/yggdrasil/issues/9)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#11](https://github.com/Unleash/yggdrasil/issues/11)**
    - Update unleash-types to 0.4.1 to deal with variant weight type response from unleash ([`6e9688a`](https://github.com/Unleash/yggdrasil/commit/6e9688a251cc7a5ce77eb6ceef41a9038b9ff0a4))
 * **[#12](https://github.com/Unleash/yggdrasil/issues/12)**
    - Bump version to 0.3.0 ([`01d5aef`](https://github.com/Unleash/yggdrasil/commit/01d5aefc52b07c2616116518b5de68d6ac36788c))
 * **[#13](https://github.com/Unleash/yggdrasil/issues/13)**
    - Remove context object from this project ([`6e73b31`](https://github.com/Unleash/yggdrasil/commit/6e73b31448a39ecf1b194fb24039f7b3d6bf533c))
 * **[#14](https://github.com/Unleash/yggdrasil/issues/14)**
    - Bump version to 0.4.0 ([`e1619d5`](https://github.com/Unleash/yggdrasil/commit/e1619d527175c197ff36bdaf9e57f6bbef0e17bf))
 * **[#15](https://github.com/Unleash/yggdrasil/issues/15)**
    - Add basic benchmarks ([`36e55c6`](https://github.com/Unleash/yggdrasil/commit/36e55c6ce33f0710e7f41f72dab86e01ddff6707))
 * **[#16](https://github.com/Unleash/yggdrasil/issues/16)**
    - Allow the engine to report the version of the Unleash client spec that it supports ([`d3f94be`](https://github.com/Unleash/yggdrasil/commit/d3f94be8ea473a8f3d465daa24bddbee487c8d45))
 * **[#17](https://github.com/Unleash/yggdrasil/issues/17)**
    - Upgrade user id strategy to string types rather than numerics ([`0a6a19b`](https://github.com/Unleash/yggdrasil/commit/0a6a19b9be93c85b095f9f1bc4f1eb8b5cfd522c))
 * **[#18](https://github.com/Unleash/yggdrasil/issues/18)**
    - Bump version to 0.4.1 ([`083f839`](https://github.com/Unleash/yggdrasil/commit/083f83994e44840647e6615164ebd17d781fd236))
 * **[#19](https://github.com/Unleash/yggdrasil/issues/19)**
    - Handle cases where get_variant is called and variants is Some but empty ([`58d1cae`](https://github.com/Unleash/yggdrasil/commit/58d1cae7fbfa78f44adccf7c098011fc047f5c9c))
 * **[#2](https://github.com/Unleash/yggdrasil/issues/2)**
    - Add readme for rust core ([`bdc912a`](https://github.com/Unleash/yggdrasil/commit/bdc912ab6a23c13d9b939152f6a7173040641ee4))
 * **[#20](https://github.com/Unleash/yggdrasil/issues/20)**
    - Chore/clippy upgrade ([`b167998`](https://github.com/Unleash/yggdrasil/commit/b167998af3fe2edd88793d0e08b8409e1c046a7b))
 * **[#21](https://github.com/Unleash/yggdrasil/issues/21)**
    - Bump types to 0.8.0 and upgrade version to 0.4.3 ([`ee13f4e`](https://github.com/Unleash/yggdrasil/commit/ee13f4ed2a6a1a13f70533fe568400bd2411742e))
 * **[#23](https://github.com/Unleash/yggdrasil/issues/23)**
    - Bump version to 0.4.4 and unleash types to 0.8.1 ([`5ec32ac`](https://github.com/Unleash/yggdrasil/commit/5ec32accf621858f74fc454d4e99616eb96b7a38))
 * **[#24](https://github.com/Unleash/yggdrasil/issues/24)**
    - Bump version to 0.4.5 ([`d996bcb`](https://github.com/Unleash/yggdrasil/commit/d996bcb37c9e0571ccc32dd130408b9b655cc406))
 * **[#25](https://github.com/Unleash/yggdrasil/issues/25)**
    - Implement metrics gathering for core ([`0b5437f`](https://github.com/Unleash/yggdrasil/commit/0b5437ff5e6417bf8c2886ed97aad77c3b5b698a))
 * **[#27](https://github.com/Unleash/yggdrasil/issues/27)**
    - Add resolve all features method ([`1617775`](https://github.com/Unleash/yggdrasil/commit/1617775d48207454bda7bc373de2bf0da7dd04cc))
 * **[#28](https://github.com/Unleash/yggdrasil/issues/28)**
    - Bump unleash types to 0.9.0 ([`1e46813`](https://github.com/Unleash/yggdrasil/commit/1e4681329b1ca5436966c22c0e892b7a024496b9))
 * **[#29](https://github.com/Unleash/yggdrasil/issues/29)**
    - Bump version to 0.5.1 ([`66bf3d8`](https://github.com/Unleash/yggdrasil/commit/66bf3d804482c0def165f2ebe282ac72c8917c57))
 * **[#30](https://github.com/Unleash/yggdrasil/issues/30)**
    - Expose project on resolved toggle ([`7f402c2`](https://github.com/Unleash/yggdrasil/commit/7f402c238163d83cd8329dc8c4cbd005c4182a9e))
 * **[#4](https://github.com/Unleash/yggdrasil/issues/4)**
    - Make clippy happy ([`0b89a94`](https://github.com/Unleash/yggdrasil/commit/0b89a94f5d91eb7ea218a7b8c502e2185c70b259))
 * **[#5](https://github.com/Unleash/yggdrasil/issues/5)**
    - Add robustness handling to context property parsing when incoming properties values have null data ([`749ac3a`](https://github.com/Unleash/yggdrasil/commit/749ac3a51076e18313b4d4b1fcff8f47b24d497f))
 * **[#6](https://github.com/Unleash/yggdrasil/issues/6)**
    - Remove reqwest, we don't use it ([`2384c16`](https://github.com/Unleash/yggdrasil/commit/2384c165aca366a6e7fdf4d7f688fd9949931471))
 * **[#7](https://github.com/Unleash/yggdrasil/issues/7)**
    - Improve sad path handling of variants ([`2b6f889`](https://github.com/Unleash/yggdrasil/commit/2b6f88924d8c7a6dd592ac816fd0a86fb223db37))
 * **[#8](https://github.com/Unleash/yggdrasil/issues/8)**
    - Add unicode support for the rule parser, this should now tolerate strings that contain unicode and a subset of punctuation ([`941bb90`](https://github.com/Unleash/yggdrasil/commit/941bb90e5c97cd14e843f55ed946005a2de811ea))
 * **[#9](https://github.com/Unleash/yggdrasil/issues/9)**
    - Bump version to 0.2.0 ([`45787e3`](https://github.com/Unleash/yggdrasil/commit/45787e38e529427bd8e351a3c69f38f84e98657f))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.2 ([`2a4a0fa`](https://github.com/Unleash/yggdrasil/commit/2a4a0faa1cbfb594e51b242818fc3f96b4ddc187))
    - Include changelog to prep for smart release ([`9b015f5`](https://github.com/Unleash/yggdrasil/commit/9b015f543b914e028b54717b2c79b4c04a8f3d8c))
    - Release unleash-yggdrasil v0.5.2 ([`e15c4a4`](https://github.com/Unleash/yggdrasil/commit/e15c4a46403461c4cedba6a2875ae7b8a075d4ee))
    - Add description and license to core ([`750e204`](https://github.com/Unleash/yggdrasil/commit/750e204ed3be475580c869c1a603c0e4da6af9bd))
    - Rename sdk-core to unleash-yggdrasil in preparation for publishing ([`d798951`](https://github.com/Unleash/yggdrasil/commit/d798951a5e34bde13974feaa8e189a5771712789))
</details>

