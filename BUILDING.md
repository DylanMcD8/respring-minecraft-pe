# Building Minecraft PE

Use an Apple silicon Mac with full Xcode selected, Python 3, CMake on PATH, and Rust installed through rustup. The device build uses Xcode 27.1, CMake 4.3.5, and Rust 1.99.0.

```sh
rustup toolchain install 1.99.0 --profile minimal
rustup target add --toolchain 1.99.0 aarch64-apple-ios aarch64-apple-ios-sim
```

Open `MinecraftPE.xcodeproj`, select the **MinecraftPE** scheme, and choose an iPhone, iPad, or ARM64 simulator. For a device, select your own development team and bundle identifier in Signing & Capabilities. Rust builds in release mode for both Xcode configurations so that normal gameplay uses the optimized interpreter. The first build compiles SDL2 and the runtime from source.

To build an unsigned device app and package it for sideloading, run from the repository root:

```sh
xcodebuild -project MinecraftPE.xcodeproj -scheme MinecraftPE \
  -configuration Release -destination 'generic/platform=iOS' \
  -derivedDataPath /tmp/MinecraftPE-build CODE_SIGNING_ALLOWED=NO
python3 Scripts/package_ipa.py \
  /tmp/MinecraftPE-build/Build/Products/Release-iphoneos/MinecraftPE.app \
  --output release-output/Minecraft-PE.ipa
```

For the Apple silicon simulator, change the destination to `generic/platform=iOS Simulator`. Use Xcode’s Run action to launch on a specific simulator.

## Source layout

- `CompatibilityProbe/ios` contains the native iOS host, Rust entry point, settings, and app metadata. `Assets.xcassets` contains the editable app icon based on the original artwork.
- `CompatibilityProbe/compatibility-core` contains the modified touchHLE runtime. The generated Minecraft translations are checked in. Routine builds do not require disassembly.
- `Vendor` contains the modified SDL2 tree and pinned Cargo dependencies. Cargo uses the repository’s relative vendor configuration and builds offline once the toolchain is installed.
- `Original/Minecraft.ipa` contains the original 0.9.0 game payload. The host loads it as a resource. The executable hash and archive provenance are in `Original/provenance.json`.
- `Scripts` contains the terrain translation generators and IPA packager. To regenerate the two Rust outputs from the supplied executable, run `python3 Scripts/regenerate.py` with Xcode selected.
