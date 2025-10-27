# Changelog - [cargo-wash](https://github.com/bircni/cargo-wash)

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

## [1.8.0](https://github.com/bircni/cargo-wash/compare/1.7.0..1.8.0) - 2025-10-27

### Features

- use condensed layout to improve table formatting in stats display - ([e37099a](https://github.com/bircni/cargo-wash/commit/e37099a63dcdb43ea53b17882a526e178c8f5853)) - Nicolas
- add progress indicators for improved user experience - ([ba7e2a6](https://github.com/bircni/cargo-wash/commit/ba7e2a643ebb4dd582e03a2d25eebbbe6d51809e)) - Nicolas
- add support for additional arguments in cargo command execution - ([68b36f1](https://github.com/bircni/cargo-wash/commit/68b36f16e686117eb508f883b517b9fd0d879e92)) - Nicolas

## [1.7.0](https://github.com/bircni/cargo-wash/compare/1.6.1..1.7.0) - 2025-10-18

### Bug Fixes

- update dependencies and exclude additional directories from package - ([211d8a2](https://github.com/bircni/cargo-wash/commit/211d8a26bb983e370ec265910102e5fe78d4746c)) - Nicolas

### Features

- refactor CLI commands and options for improved functionality - ([9bcb3c1](https://github.com/bircni/cargo-wash/commit/9bcb3c1838165a7ea95097098ada50c3c5340f08)) - Nicolas

## [1.6.1](https://github.com/bircni/cargo-wash/compare/1.6.0..1.6.1) - 2025-08-17

### Continuous Integration

- Update Rust toolchain version to stable in CI and deploy workflows - ([3e6b620](https://github.com/bircni/cargo-wash/commit/3e6b620a8fe9465e9aa7ffa08cf5e1b4ca95fc85)) - Nicolas

### Refactoring

- Rename check_project to get_project and improve project validation logic - ([60122b8](https://github.com/bircni/cargo-wash/commit/60122b8011a76bd4022264b27a16b2431b64f821)) - Nicolas
- Update path handling in ExecuteOptions and Options to use non-optional PathBuf - ([ae35446](https://github.com/bircni/cargo-wash/commit/ae35446d632a891f80b447221630d7d17fb06d94)) - Nicolas
- Update path handling in utility and options to use references for PathBuf - ([d56d079](https://github.com/bircni/cargo-wash/commit/d56d0799d796732d37c424255d6471f2444f3286)) - Nicolas

### Tests

- fix tests for new structure - ([e747fa6](https://github.com/bircni/cargo-wash/commit/e747fa6250b541d630d628b671a535693a716a8d)) - Nicolas

### Build

- Update dependencies and remove unused build script - ([b6368bd](https://github.com/bircni/cargo-wash/commit/b6368bd4a98f8f7eccc9c6aee28b955dfe9b60bf)) - Nicolas

## [1.6.0](https://github.com/bircni/cargo-wash/compare/1.5.0..1.6.0) - 2025-06-19

### Bug Fixes

- Rename rebuilt_projects to processed_projects for clarity - ([efefd44](https://github.com/bircni/cargo-wash/commit/efefd44d747b1071f36bfef5192de76167534079)) - Nicolas

### Features

- Refactor command handling and add support for executing various cargo commands - ([49f91da](https://github.com/bircni/cargo-wash/commit/49f91da9b312895db65c9960a49d86ce8090414a)) - Nicolas
- Refactor command options and enhance command handling - ([76bb31e](https://github.com/bircni/cargo-wash/commit/76bb31e9ff712b0aa5ea738e284c9bee6248b195)) - Nicolas

### Build

- update toolchain to 1.87 and bump dependencies versions - ([504a470](https://github.com/bircni/cargo-wash/commit/504a4704132c5333689010c84580508d9f3ab0d8)) - Nicolas
- fix linux deploy targets - ([315e12a](https://github.com/bircni/cargo-wash/commit/315e12a62a252d7649871d34db9c04cf08e8e9ea)) - Nicolas

## [1.5.0](https://github.com/bircni/cargo-wash/compare/1.4.0..1.5.0) - 2025-06-08

### Features

- Enhanced logging & added an update check. ([#18](https://github.com/bircni/cargo-wash/issues/18)) - ([c55b489](https://github.com/bircni/cargo-wash/commit/c55b489e786a7586e4b0de2dde128fa7fdaf3772)) - Nicolas

### Build

- **(dependabot)** group dependency bumps - ([05439aa](https://github.com/bircni/cargo-wash/commit/05439aaa710e2482fd3f55450ec8cc162d38b0eb)) - Nicolas

## [1.4.0](https://github.com/bircni/cargo-wash/compare/1.3.0..1.4.0) - 2025-05-31

### Features

- rebuild all rust projects ([#14](https://github.com/bircni/cargo-wash/issues/14)) - ([e1616ca](https://github.com/bircni/cargo-wash/commit/e1616ca774e09d19e339e61d3aa87575c1323407)) - Nicolas

### Build

- update to rust 1.87 ([#13](https://github.com/bircni/cargo-wash/issues/13)) - ([13e3c58](https://github.com/bircni/cargo-wash/commit/13e3c582ab87c9a7107204e4c214b0802a293de4)) - Nicolas

## [1.3.0](https://github.com/bircni/cargo-wash/compare/1.2.2..1.3.0) - 2025-05-10

### Features

- remove additonal target folder - to comply with cargo clean - ([35a171f](https://github.com/bircni/cargo-wash/commit/35a171f8324a0196f24b4026d29a4b9c38e17ba3)) - Nicolas
- remove dry-run option as this was not working correctly and not being used - ([f7af551](https://github.com/bircni/cargo-wash/commit/f7af551f063dc9934fa2a38dfa30f2eec8787044)) - Nicolas

### Build

- **(deps)** bump insta from 1.43.0 to 1.43.1 ([#12](https://github.com/bircni/cargo-wash/issues/12)) - ([6bfc5de](https://github.com/bircni/cargo-wash/commit/6bfc5deba56c18037542176f139b435d8f525bd7)) - dependabot[bot]

## [1.2.2](https://github.com/bircni/cargo-wash/compare/1.2.1..1.2.2) - 2025-04-26

### Bug Fixes

- typos check - ([244505a](https://github.com/bircni/cargo-wash/commit/244505a6048f75b1f54656faa6af6af96f5edca9)) - Nicolas

### Documentation

- add Cargo.lock to release scripts - ([59a15ad](https://github.com/bircni/cargo-wash/commit/59a15ad656991d60bcddf4b67e419cc544e6f564)) - Nicolas
- push with follow tags in release script - ([9a0303d](https://github.com/bircni/cargo-wash/commit/9a0303dc32d1d73620852f752b70e63abe7fc97c)) - Nicolas

### Linting

- apply correct lints - ([33caceb](https://github.com/bircni/cargo-wash/commit/33cacebd4cc9768106a24bc1385b7ca6d69d1efb)) - Nicolas

### Refactoring

- rework and move files - ([4ab9fcb](https://github.com/bircni/cargo-wash/commit/4ab9fcb378c850b88cda7497661706b3a5bd5424)) - Nicolas

## [1.2.1](https://github.com/bircni/cargo-wash/compare/1.2.0..1.2.1) - 2025-04-20

### Bug Fixes

- exclude options not correctly parsed - ([b13ddea](https://github.com/bircni/cargo-wash/commit/b13ddea365ab59b7f89ebeffc98db394daedb551)) - Nicolas
- typos - ([7dc245c](https://github.com/bircni/cargo-wash/commit/7dc245c031e2b006639a7d83b04249153dd3ca23)) - Nicolas

### Continuous Integration

- only run tests on ubuntu & use rust 1.86 - ([a2a7576](https://github.com/bircni/cargo-wash/commit/a2a7576de12ed61af590891e15aabd8cbceb284b)) - Nicolas

### Documentation

- add CI option to Changelog Generator - ([82504d1](https://github.com/bircni/cargo-wash/commit/82504d166f8a77098d73fdaf3d4949626835a0b0)) - Nicolas

## [1.2.0](https://github.com/bircni/cargo-wash/compare/1.1.0..1.2.0) - 2025-04-18

### Documentation

- **(README)** fix usage and add installation instructions - ([eb937b1](https://github.com/bircni/cargo-wash/commit/eb937b1e58f8ac46b5ca751c932de5059f37dc13)) - Nicolas

### Features

- add release scripts - ([9df5c27](https://github.com/bircni/cargo-wash/commit/9df5c27f66d698616efa4f2bfa52a1863eb6f2d3)) - Nicolas

### Miscellaneous Chores

- **(build-script)** set the version according to tags - ([193e3af](https://github.com/bircni/cargo-wash/commit/193e3afc2400631bf651650bd7e3d90a700ad841)) - Nicolas

## [1.1.0](https://github.com/bircni/cargo-wash/compare/1.0.0..1.1.0) - 2025-03-28

### Features

- **(options)** add exclude and additional build folder ([#11](https://github.com/bircni/cargo-wash/issues/11)) - ([5c8bd98](https://github.com/bircni/cargo-wash/commit/5c8bd9874854e23f4822bc3a1483af7abd037a75)) - Nicolas

## [1.0.0](https://github.com/bircni/cargo-wash/compare/0.2.5..1.0.0) - 2025-03-26

### Miscellaneous Chores

- **(refactoring)** removing unused features and enhancing the tool ([#10](https://github.com/bircni/cargo-wash/issues/10)) - ([9e3abb2](https://github.com/bircni/cargo-wash/commit/9e3abb2e3a96c9c7cbb4567441ff9fad7748787b)) - Nicolas

## [0.1.0-beta.1] - 2024-09-28
