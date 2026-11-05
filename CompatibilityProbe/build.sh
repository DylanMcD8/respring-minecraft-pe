#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "$0")" && pwd -P)
repository=$(cd "$root/.." && pwd -P)
cd "$repository"
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
sdk=${1:-iphonesimulator}
case "$sdk" in
  iphoneos) target=aarch64-apple-ios; platform=ios ;;
  iphonesimulator) target=aarch64-apple-ios-sim; platform=ios-simulator ;;
  *) exit 2 ;;
esac
build=${CHUZZLE_NATIVE_BUILD:-"$repository/.build"}
export CARGO_TARGET_DIR="$build/rust"
export IPHONEOS_DEPLOYMENT_TARGET=15.0
export CMAKE_POLICY_VERSION_MINIMUM=3.5
export SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
sdl="$build/sdl/$sdk/install"
if [ ! -f "$sdl/lib/libSDL2.a" ]; then
  CHUZZLE_SDL_BUILD="$build/sdl" bash "$root/rebuild_sdl.sh" "$sdk"
fi
export CHUZZLE_SDL_LIBRARY="$sdl/lib/libSDL2.a"
unset RUSTFLAGS
rust_flags=(-L "native=$sdl/lib" --remap-path-prefix "$HOME=~" --remap-path-prefix "$repository=.")
frameworks=(UIKit Foundation OpenGLES OpenAL QuartzCore AVFoundation AudioToolbox CoreAudio CoreGraphics CoreMotion GameController CoreHaptics CoreBluetooth Metal Security)
link=()
for framework in "${frameworks[@]}"; do
  rust_flags+=(-C link-arg=-framework -C "link-arg=$framework")
  link+=(-framework "$framework")
done
printf -v CARGO_ENCODED_RUSTFLAGS '%s\x1f' "${rust_flags[@]}"
export CARGO_ENCODED_RUSTFLAGS=${CARGO_ENCODED_RUSTFLAGS%$'\x1f'}
cargo build --manifest-path "$root/ios/rust-entry/Cargo.toml" --target "$target" --release --offline --locked
app="$build/$sdk/MinecraftPE.app"
mkdir -p "$app"
xcrun --sdk "$sdk" clang -ffile-prefix-map="$HOME=~" -arch arm64 -isysroot "$SDKROOT" \
  -target "arm64-apple-$platform" -mios-version-min=15.0 \
  -I "$sdl/include/SDL2" "$root/ios/main.m" "$CARGO_TARGET_DIR/$target/release/libminecraft_probe.a" \
  -L "$sdl/lib" -lSDL2main -lSDL2 -lc++ -liconv -lz "${link[@]}" \
  -Wl,-no_adhoc_codesign -o "$app/MinecraftProbe"
cp "$repository/Original/Minecraft.ipa" "$app/"
ditto "$root/ios/Settings.bundle" "$app/Settings.bundle"
for folder in touchHLE_fonts touchHLE_dylibs; do
  ditto "$root/compatibility-core/$folder" "$app/$folder"
done
cp "$root/compatibility-core/touchHLE_default_options.txt" "$app/"
ditto "$repository/Notices" "$app/ThirdPartyNotices"
