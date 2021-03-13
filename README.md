# Evo

Evo aims to simulate evolution of simple digital organisms. It still has a long way to go. This "rust-evo" repo is a Rust rewrite and extension of an older Java version in my "Evo" repo.

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

### Running

Run evo's latest. Often unexciting looking. Hit Esc (or q or x) to exit.

```
cargo run --release
```

Run something that looks cool.

```
git tag
git checkout nice_main_2     # or other promising-looking tag
cargo run --release
```

Run the "duckweed" example.

```
cargo run --example duckweed --release
```

Actions while evo is running.

```
Esc,q,x         - exit
p               - pause (toggle)
t               - single tick
f               - fast forward (toggle)
click on cell   - select for debug output (toggle)
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

* **Franz Amador** - *Primary*
