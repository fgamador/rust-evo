# Evo

Evo aims to simulate evolution of simple digital organisms.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) - The Rust compiler and supporting tools

### Installing

[Fork and clone](https://akrabat.com/the-beginners-guide-to-contributing-to-a-github-project/) this repo.

Run the tests.

```
cargo test
```

Run evo's latest. Is often boring looking.

```
cargo run --release
```

Run something that looks cool.

```
git tag
git checkout nice_main_2     # or other promising-looking tag
cargo run --release
```

### Development Tooling

* [rustfmt](https://github.com/rust-lang/rustfmt) - The Rust standard code formatter
```
rustup component add rustfmt
```

* [Clippy](https://github.com/rust-lang/rust-clippy) - The Rust standard code linter
```
rustup component add clippy
```

* A development environment, such as [IntelliJ IDEA](https://www.jetbrains.com/idea/download) with the [Rust plugin](https://intellij-rust.github.io/), or one of the ones listed in "Other tools" [here](https://www.rust-lang.org/learn/get-started)

## Authors

* **Franz Amador** - *Initial work*
