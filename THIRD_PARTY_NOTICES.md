# Third-party sources and notices

- The modified [touchHLE runtime](https://github.com/touchHLE/touchHLE) and ARM interpreter carry MPL-2.0 notices. Source and license texts are in `CompatibilityProbe/compatibility-core`.
- The modified [SDL2](https://github.com/libsdl-org/SDL) source is in `Vendor/SDL2`, with its upstream zlib license. Rust library sources and licenses are in `Vendor/Rust`, pinned by `CompatibilityProbe/ios/rust-entry/Cargo.lock`.
- stb_image, PVRTDecompress, the native zlib 1.1.3 decoder, and the fonts retain the notices beside their source or data files. Additional dependency notices are in `Notices`.
- Legacy guest libraries are in `CompatibilityProbe/compatibility-core/touchHLE_dylibs`. Its README records their provenance. This restoration also uses the ARM libc++.1 and libc++abi libraries from the iOS 7.1 SDK. The release’s runtime sources download contains the upstream GCC and libstdc++ archives for the bundled GNU binaries.
- Minecraft, its executable, textures, audio, fonts, interface, and original icon belong to their respective rights holders. The project licenses apply to the restoration code as described in [LICENSE.md](LICENSE.md).
