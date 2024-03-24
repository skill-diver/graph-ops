# Contribution guidelines

We welcome and encourage contributions of all kinds, such as:

- Tickets with issue reports of feature requests
- Documentation improvements
- Code (PR or PR Review)

## Setup the environment

1. Setup the Rust toolchain. See [the Rust book](https://doc.rust-lang.org/book/ch01-01-installation.html) for details.
2. Install Protobuf compiler. See [the Protobuf book](https://developers.google.com/protocol-buffers/docs/downloads) for details.
3. Setup neo4j (via default port 7687). See [the neo4j documentation](https://neo4j.com/docs/operations-manual/current/installation/) for details.
4. Setup Etcd (via default port 2379). See [the etcd documentation](https://etcd.io/docs/latest/install/) for details.
5. Setup Redis (via port 7379). See [the redis documentation](https://redis.io/docs/getting-started/) for details.
6. Check the configurations in `examples/quickstart/ofnil.toml` and `examples/quickstart/.env`.
7. Build and test

- `cargo build`
- `cargo test`
- `OFNIL_HOME=examples/quickstart RUST_LOG=info cargo run --example quickstart`.

## Test and Check

Contributors are encouraged to add unit tests (#[test] or #[tokio::test]) for the project.

### Tests

To run the tests, use the following command:

```bash
cargo test
```

### Code Style

Contributors are encouraged to follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html).

Before submitting a PR, please run the following command to format the code and resolve potential problems:

```bash
cargo fmt
cargo clippy
```

## Submit a PR

### PR Title Convention

A valid PR title should begin with one of [the following prefixes](https://github.com/commitizen/conventional-commit-types/blob/master/index.json):

- `feat`: A new feature
- `fix`: A bug fix
- `doc`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
- `ci`: Changes to CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files
- `revert`: Reverts a previous commit

You are encourage to check out and review [previous PRs](https://github.com/ofnil/ofnil/pulls).

### PR Description

A PR description should include `What's changed and what's your intention` section. The more detailed the description is, the easier it is for other community members to review and merge your PR.

### Sign the CLA

Contributors will need to sign Ofnil's CLA.
