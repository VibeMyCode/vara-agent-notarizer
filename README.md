## The **vara-agent-notarizer-tmp** program

[![Build Status](https://github.com/gear-tech/vara-agent-notarizer-tmp/workflows/CI/badge.svg)](https://github.com/gear-tech/vara-agent-notarizer-tmp/actions)

Program **vara-agent-notarizer-tmp** for [⚙️ Gear Protocol](https://github.com/gear-tech/gear) written in [⛵ Sails](https://github.com/gear-tech/sails) framework.

The program workspace includes the following packages:
- `vara-agent-notarizer-tmp` is the package allowing to build WASM binary for the program and IDL file for it.
  The package also includes integration tests for the program in the `tests` sub-folder
- `vara-agent-notarizer-tmp-app` is the package containing business logic for the program represented by the `VaraAgentNotarizerTmp` structure.
- `vara-agent-notarizer-tmp-client` is the package containing the client for the program allowing to interact with it from another program, tests, or off-chain client.

### 🏗️ Building

```bash
cargo build --release
```

### ✅ Testing

```bash
cargo test --release
```

# License

The source code is licensed under the [MIT license](LICENSE).
