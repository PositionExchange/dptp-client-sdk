# Plugin: https://pub.dev/packages/dptp_client_sdk

## dptp_client_sdk

SDK for DPTP FuturX using Rust logic

## Getting Started

- Setting Android 
  - Copy folder `lib/jniLibs` to `android/app/src/main` or run cmd `cp -R {sdk-lib-path}/lib/android/jniLibs android/app/src/main &&`

- Setting iOS
  - Copy folder `lib/ios` to `ios/Runner` or run cmd `cp -R {sdk-lib-path}/lib/dptp_client_sdk-0.0.2/lib/ios`
  - Add this line to `ios/Runner/Runner-Bridging-Header.h`:
  ```+#import "bridge_generated.h"```
  - And in `ios/Runner/AppDelegate.swift`: `print("dummy_value=\(dummy_method_to_enforce_bundling())");`
  - Open `ios/Runner/` 
  - Add `libdptp_sdk.a` to `Runner/Frameworks` 
  - Select `Targets/Runner` -> `Build Phases` tab -> Expand the `Link Binary With Libraries` phase, and add `libdptp_sdk.a`
