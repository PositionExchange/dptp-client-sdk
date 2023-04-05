# dptp-client-sdk

This SDK provides a simple and effective way to interact with DPTP smart contracts. It is designed to work with both web application (through WebAssembly bindings) and Flutter (using the Flutter Rust Bridge).

## Structure

The project structure is organized as follows:

```bash
dptp-client-sdk/
│
├── core/                       # Core Rust library
│   ├── src/
│   │   ├── lib.rs              # Main entry point
│   │   ├── contract.rs         # Contract interaction logic
│   │   └── utils.rs            # Utility functions
│   ├── Cargo.toml              # Rust dependencies
│   └── README.md               # Core library documentation
│
├── wasm/                       # WebAssembly bindings for React.js
│   ├── src/
│   │   ├── lib.rs              # Main entry point for WASM bindings
│   │   └── utils.rs            # Utility functions for WASM
│   ├── Cargo.toml              # Rust dependencies for WASM
│   └── README.md               # WASM bindings documentation
│
└── flutter/                    # Flutter bindings using Flutter Rust Bridge
    ├── lib/
    │   ├── dptp_client_sdk.dart          # Dart bindings for the Rust library
    │   └── dptp_client_sdk_ffi.dart      # Dart FFI for Rust functions
    ├── rust/
    │   ├── src/
    │   │   ├── lib.rs          # Main entry point for Flutter Rust Bridge
    │   │   └── utils.rs        # Utility functions for Flutter
    │   └── Cargo.toml          # Rust dependencies for Flutter
    ├── pubspec.yaml            # Dart dependencies
    └── README.md               # Flutter bindings documentation
```
## Usage

### Web Application

To use the SDK in a Web (React, Vue,...) project, first, build the WASM bindings:

```bash
cd wasm
cargo build --target wasm32-web --release
```
Then, include the generated .wasm file in your React.js project and import it as a module.

Flutter
To use the SDK in a Flutter project, you will need to include the flutter folder in your project as a package. Add the following to your pubspec.yaml:

```yaml
dependencies:
  dptp_client_sdk:
    path: path/to/flutter/folder
```

Then, import and use the SDK in your Dart code:

```dart
import 'package:dptp_client_sdk/dptp_client_sdk.dart';
```
## Documentation

Please refer to the README files in each subdirectory for detailed information about each component:

- Core Rust library (core/README.md)
- WebAssembly bindings for React.js (wasm/README.md)
- Flutter bindings (flutter/README.md)
