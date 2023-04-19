# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.5.4 (2023-04-19)

### Other

 - <csr-id-0c1edd38338e9ad0659ed80218aabd8235c36899/> Bump unleash types to 0.10.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 46 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#33](https://github.com/sighphyre/yggdrasil/issues/33)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#33](https://github.com/sighphyre/yggdrasil/issues/33)**
    - Bump unleash types to 0.10.0 ([`0c1edd3`](https://github.com/sighphyre/yggdrasil/commit/0c1edd38338e9ad0659ed80218aabd8235c36899))
</details>

## v0.5.3 (2023-03-03)

### New Features

 - <csr-id-6f8bd369287000d248608a3ac4b2d062326c8cab/> add support for variants overrides on arbitrary context fields

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#31](https://github.com/sighphyre/yggdrasil/issues/31)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#31](https://github.com/sighphyre/yggdrasil/issues/31)**
    - Add support for variants overrides on arbitrary context fields ([`6f8bd36`](https://github.com/sighphyre/yggdrasil/commit/6f8bd369287000d248608a3ac4b2d062326c8cab))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.3 ([`b58f062`](https://github.com/sighphyre/yggdrasil/commit/b58f06258e4f8540a93afe9dd3b5f264d84adc77))
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
 - 25 unique issues were worked on: [#11](https://github.com/sighphyre/yggdrasil/issues/11), [#12](https://github.com/sighphyre/yggdrasil/issues/12), [#13](https://github.com/sighphyre/yggdrasil/issues/13), [#14](https://github.com/sighphyre/yggdrasil/issues/14), [#15](https://github.com/sighphyre/yggdrasil/issues/15), [#16](https://github.com/sighphyre/yggdrasil/issues/16), [#17](https://github.com/sighphyre/yggdrasil/issues/17), [#18](https://github.com/sighphyre/yggdrasil/issues/18), [#19](https://github.com/sighphyre/yggdrasil/issues/19), [#2](https://github.com/sighphyre/yggdrasil/issues/2), [#20](https://github.com/sighphyre/yggdrasil/issues/20), [#21](https://github.com/sighphyre/yggdrasil/issues/21), [#23](https://github.com/sighphyre/yggdrasil/issues/23), [#24](https://github.com/sighphyre/yggdrasil/issues/24), [#25](https://github.com/sighphyre/yggdrasil/issues/25), [#27](https://github.com/sighphyre/yggdrasil/issues/27), [#28](https://github.com/sighphyre/yggdrasil/issues/28), [#29](https://github.com/sighphyre/yggdrasil/issues/29), [#30](https://github.com/sighphyre/yggdrasil/issues/30), [#4](https://github.com/sighphyre/yggdrasil/issues/4), [#5](https://github.com/sighphyre/yggdrasil/issues/5), [#6](https://github.com/sighphyre/yggdrasil/issues/6), [#7](https://github.com/sighphyre/yggdrasil/issues/7), [#8](https://github.com/sighphyre/yggdrasil/issues/8), [#9](https://github.com/sighphyre/yggdrasil/issues/9)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#11](https://github.com/sighphyre/yggdrasil/issues/11)**
    - Update unleash-types to 0.4.1 to deal with variant weight type response from unleash ([`6e9688a`](https://github.com/sighphyre/yggdrasil/commit/6e9688a251cc7a5ce77eb6ceef41a9038b9ff0a4))
 * **[#12](https://github.com/sighphyre/yggdrasil/issues/12)**
    - Bump version to 0.3.0 ([`01d5aef`](https://github.com/sighphyre/yggdrasil/commit/01d5aefc52b07c2616116518b5de68d6ac36788c))
 * **[#13](https://github.com/sighphyre/yggdrasil/issues/13)**
    - Remove context object from this project ([`6e73b31`](https://github.com/sighphyre/yggdrasil/commit/6e73b31448a39ecf1b194fb24039f7b3d6bf533c))
 * **[#14](https://github.com/sighphyre/yggdrasil/issues/14)**
    - Bump version to 0.4.0 ([`e1619d5`](https://github.com/sighphyre/yggdrasil/commit/e1619d527175c197ff36bdaf9e57f6bbef0e17bf))
 * **[#15](https://github.com/sighphyre/yggdrasil/issues/15)**
    - Add basic benchmarks ([`36e55c6`](https://github.com/sighphyre/yggdrasil/commit/36e55c6ce33f0710e7f41f72dab86e01ddff6707))
 * **[#16](https://github.com/sighphyre/yggdrasil/issues/16)**
    - Allow the engine to report the version of the Unleash client spec that it supports ([`d3f94be`](https://github.com/sighphyre/yggdrasil/commit/d3f94be8ea473a8f3d465daa24bddbee487c8d45))
 * **[#17](https://github.com/sighphyre/yggdrasil/issues/17)**
    - Upgrade user id strategy to string types rather than numerics ([`0a6a19b`](https://github.com/sighphyre/yggdrasil/commit/0a6a19b9be93c85b095f9f1bc4f1eb8b5cfd522c))
 * **[#18](https://github.com/sighphyre/yggdrasil/issues/18)**
    - Bump version to 0.4.1 ([`083f839`](https://github.com/sighphyre/yggdrasil/commit/083f83994e44840647e6615164ebd17d781fd236))
 * **[#19](https://github.com/sighphyre/yggdrasil/issues/19)**
    - Handle cases where get_variant is called and variants is Some but empty ([`58d1cae`](https://github.com/sighphyre/yggdrasil/commit/58d1cae7fbfa78f44adccf7c098011fc047f5c9c))
 * **[#2](https://github.com/sighphyre/yggdrasil/issues/2)**
    - Add readme for rust core ([`bdc912a`](https://github.com/sighphyre/yggdrasil/commit/bdc912ab6a23c13d9b939152f6a7173040641ee4))
 * **[#20](https://github.com/sighphyre/yggdrasil/issues/20)**
    - Chore/clippy upgrade ([`b167998`](https://github.com/sighphyre/yggdrasil/commit/b167998af3fe2edd88793d0e08b8409e1c046a7b))
 * **[#21](https://github.com/sighphyre/yggdrasil/issues/21)**
    - Bump types to 0.8.0 and upgrade version to 0.4.3 ([`ee13f4e`](https://github.com/sighphyre/yggdrasil/commit/ee13f4ed2a6a1a13f70533fe568400bd2411742e))
 * **[#23](https://github.com/sighphyre/yggdrasil/issues/23)**
    - Bump version to 0.4.4 and unleash types to 0.8.1 ([`5ec32ac`](https://github.com/sighphyre/yggdrasil/commit/5ec32accf621858f74fc454d4e99616eb96b7a38))
 * **[#24](https://github.com/sighphyre/yggdrasil/issues/24)**
    - Bump version to 0.4.5 ([`d996bcb`](https://github.com/sighphyre/yggdrasil/commit/d996bcb37c9e0571ccc32dd130408b9b655cc406))
 * **[#25](https://github.com/sighphyre/yggdrasil/issues/25)**
    - Implement metrics gathering for core ([`0b5437f`](https://github.com/sighphyre/yggdrasil/commit/0b5437ff5e6417bf8c2886ed97aad77c3b5b698a))
 * **[#27](https://github.com/sighphyre/yggdrasil/issues/27)**
    - Add resolve all features method ([`1617775`](https://github.com/sighphyre/yggdrasil/commit/1617775d48207454bda7bc373de2bf0da7dd04cc))
 * **[#28](https://github.com/sighphyre/yggdrasil/issues/28)**
    - Bump unleash types to 0.9.0 ([`1e46813`](https://github.com/sighphyre/yggdrasil/commit/1e4681329b1ca5436966c22c0e892b7a024496b9))
 * **[#29](https://github.com/sighphyre/yggdrasil/issues/29)**
    - Bump version to 0.5.1 ([`66bf3d8`](https://github.com/sighphyre/yggdrasil/commit/66bf3d804482c0def165f2ebe282ac72c8917c57))
 * **[#30](https://github.com/sighphyre/yggdrasil/issues/30)**
    - Expose project on resolved toggle ([`7f402c2`](https://github.com/sighphyre/yggdrasil/commit/7f402c238163d83cd8329dc8c4cbd005c4182a9e))
 * **[#4](https://github.com/sighphyre/yggdrasil/issues/4)**
    - Make clippy happy ([`0b89a94`](https://github.com/sighphyre/yggdrasil/commit/0b89a94f5d91eb7ea218a7b8c502e2185c70b259))
 * **[#5](https://github.com/sighphyre/yggdrasil/issues/5)**
    - Add robustness handling to context property parsing when incoming properties values have null data ([`749ac3a`](https://github.com/sighphyre/yggdrasil/commit/749ac3a51076e18313b4d4b1fcff8f47b24d497f))
 * **[#6](https://github.com/sighphyre/yggdrasil/issues/6)**
    - Remove reqwest, we don't use it ([`2384c16`](https://github.com/sighphyre/yggdrasil/commit/2384c165aca366a6e7fdf4d7f688fd9949931471))
 * **[#7](https://github.com/sighphyre/yggdrasil/issues/7)**
    - Improve sad path handling of variants ([`2b6f889`](https://github.com/sighphyre/yggdrasil/commit/2b6f88924d8c7a6dd592ac816fd0a86fb223db37))
 * **[#8](https://github.com/sighphyre/yggdrasil/issues/8)**
    - Add unicode support for the rule parser, this should now tolerate strings that contain unicode and a subset of punctuation ([`941bb90`](https://github.com/sighphyre/yggdrasil/commit/941bb90e5c97cd14e843f55ed946005a2de811ea))
 * **[#9](https://github.com/sighphyre/yggdrasil/issues/9)**
    - Bump version to 0.2.0 ([`45787e3`](https://github.com/sighphyre/yggdrasil/commit/45787e38e529427bd8e351a3c69f38f84e98657f))
 * **Uncategorized**
    - Release unleash-yggdrasil v0.5.2 ([`2a4a0fa`](https://github.com/sighphyre/yggdrasil/commit/2a4a0faa1cbfb594e51b242818fc3f96b4ddc187))
    - Include changelog to prep for smart release ([`9b015f5`](https://github.com/sighphyre/yggdrasil/commit/9b015f543b914e028b54717b2c79b4c04a8f3d8c))
    - Release unleash-yggdrasil v0.5.2 ([`e15c4a4`](https://github.com/sighphyre/yggdrasil/commit/e15c4a46403461c4cedba6a2875ae7b8a075d4ee))
    - Add description and license to core ([`750e204`](https://github.com/sighphyre/yggdrasil/commit/750e204ed3be475580c869c1a603c0e4da6af9bd))
    - Rename sdk-core to unleash-yggdrasil in preparation for publishing ([`d798951`](https://github.com/sighphyre/yggdrasil/commit/d798951a5e34bde13974feaa8e189a5771712789))
</details>

