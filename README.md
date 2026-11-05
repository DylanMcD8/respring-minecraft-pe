<p align="center">
  <img src="docs/images/RespringHeader.webp" width="800" alt="Respring">
</p>

# Respring – Minecraft PE
The classic Minecraft: Pocket Edition (v0.9.0, pre-Bedrock), restored with the help of Codex (GPT-6 Astra) for modern iPhones and iPads as part of project Respring!

### TO DO!!
- [ ] Repo header!
- [ ] Update AI disclosures based on Michael’s contributions
- [ ] Add credit to Michael for his contributions
- [ ] Clean up settings
- [ ] Remove FPS counter (or make it off by default at least)
- [ ] Fix portrait/landscape scaling issue
- [ ] Test out iPhone / iPad layouts on multiple devices to choose the best layouts for Automatic mode
- [ ] Review and cleanup all settings
- [ ] Fix upside down launch screen
- [ ] Compile work report and update skill
- [ ] Update skill with instructions for other users to create and share work report
- [x] Remaster the app icon
- [ ] Add to Respring website once all else is complete!

### How does it work?
The original 2014 game runs on a modified touchHLE runtime. It interprets ARM/Thumb instructions and runs selected terrain, lighting, and rendering routines as native code after checking them against the original game. A native iOS host handles the display, touch input, and audio. No JIT is required!

### Enhancements
- Full Resolution: The entire app will render at your device’s full resolution and aspect ratio!
- Choose Your Device Interface: You can switch between a 3.5-inch iPhone, 4-inch iPhone, or iPad. The game will use the full resolution and aspect ratio regardless, but certain UI elements may work better depending on your actual device. The app will, by default, select the one that will probably work best.
- Unlock Frame Rate: Pick your maximum frame rate, up to 120 FPS on ProMotion displays! (frame rate may drop during certain actions/sequences)
- Background / Multitasking: when you leave the app, the game will no longer relaunch, so you can switch between tasks or open Control Center while playing.
- App Icon: I have "remastered" the app icon with Icon Composer with full support for Default, Dark, Clear, and Tinted modes.

### Notes
- The app enhancements can be enabled or disabled in Settings → Apps → Minecraft PE.
- New chunks may take some time to appear (especially when flying), but don’t worry, they will appear!
- When creating a new world, open the Advanced menu to enable an infinite world (the game defaults to a fixed-size world).
- It's not perfect! There may still be crashes. If you experience a crash or some other issue, please open an Issue on this repository.

As stated before, this project was achieved through the usage of generative AI. Outside of this README and Releases release notes, most things were written by an AI agent.

## How to Install
Download **Minecraft-PE.ipa** from [Releases](https://github.com/DylanMcD8/respring-minecraft-pe/releases). It is an unsigned ARM64 device app. Sign it for your iPhone or iPad with your preferred sideloading tool, such as [Sideloadly](https://sideloadly.io).

Alternatively, open **MinecraftPE.xcodeproj** and build with your own Apple development team. The deployment target is iOS/iPadOS 15.0. Restoration testing included the physical iPad mini (A17 Pro), iPhone Duo, and iPhone Air, and the iOS 27.1 Duo simulator.

## Build from Source
*Written by AI agent*

1. Install the pinned Rust toolchain, ARM64 targets, and CMake on an Apple silicon Mac with full Xcode. See [BUILDING.md](BUILDING.md).
2. Open `MinecraftPE.xcodeproj`, select the **MinecraftPE** scheme, and choose your destination.
3. For a physical device, select your development team and an available bundle identifier. Build and run.

The repository includes the original game payload, the modified runtime, and the dependencies for the offline Cargo build.

## Credits and Licensing
Minecraft: Pocket Edition was created by Mojang. Minecraft and its original game assets belong to their respective rights holders. Respring is an unofficial preservation project.

The compatibility runtime derives from [touchHLE](https://github.com/touchHLE/touchHLE). The runtime, SDL2, and other dependencies retain their respective licenses. See [LICENSE.md](LICENSE.md) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
