# Performance notes

Status of the frame-rate work on the CompatibilityProbe runtime, what landed, how it
was measured, and what is left. Written so the next pass can start from here cold.

## How to measure

- `MINECRAFT_PROFILE=1` writes `Documents/minecraft-profile.jsonl` in the app
  container. Every 5 s it emits a `window` event (guest/host fps, p95 frame time,
  per-phase ms, `interpreter_retired`) and, with background terrain, a
  `native_refresh_window` event from `main.m` (`callbacks`, `distinct_images`).
- `PEGGLE_PERF_INTERPRETER_TIMING=1` adds interpreter phase timing;
  `MINECRAFT_CENSUS=1` histograms interpreted PCs.
- Simulator launch with profiling:
  `SIMCTL_CHILD_MINECRAFT_PROFILE=1 xcrun simctl launch <udid> com.respring.minecraftpe.probe`.
- `distinct_images` counts display-link ticks on which a new image was submitted.
  Before the composite skip (below) every tick re-submitted the same composite, so
  it read 300/5 s even when the guest repeated a frame. It now counts real new
  frames, and so does the FPS label. Compare like with like.

The simulator's GL is Apple's software renderer and reports a 60 Hz maximum, so
sim numbers are GL-fill-bound and can't show 120 Hz. Use them for regressions and
interpreter throughput, and verify the frame-rate claims on a ProMotion device.
With Native Render Resolution on (the default), the blurred title-screen panorama
runs at about 11 FPS on the iPhone 17 Pro simulator, on HEAD (10.4) and now (11.0)
alike. That is software fill at 2622×1206, not a pacing problem.

## Where the time goes (simulator, in a world, steady state)

| Phase | Share of main thread |
|---|---|
| GL (software renderer) | ~52% |
| Compositor | ~18% |
| Interpreter | ~10% (~375k guest instructions per frame, ~200 MIPS) |

On device, GL moves to the GPU and the interpreter and compositor dominate, which
is why most of the work below targets those.

## What landed

### Pacing and scheduling (largest visible win)
- `libc/sched.rs`: `sched_yield` now yields the coroutine. The terrain worker
  handshake spins on it, and returning 0 without yielding burned whole 100k-instruction
  quanta.
- `environment.rs` `schedule_next_thread`: condition-variable timeouts are folded
  into the wake-up deadline, so a timed wait no longer oversleeps to the next
  unrelated wake. After a host sleep, the scheduler pumps SDL events
  (`Window::poll_for_events_now`) so the UIKit display link, which only ticks inside
  `SDL_PumpEvents`, isn't starved while every guest thread is blocked.
- `window.rs`: poll throttle raised to 1/240 s so 120 Hz ticks aren't coalesced.
- `main.m` `minecraftConfigureRefresh`: the display-link observer is installed for
  background terrain modes even with profiling and the FPS label off, so the guest
  follows the link's vsync deadlines instead of the NSTimer path.
- `main.m` `registerDefaults` + `Settings.bundle/Root.plist`: TerrainProcessing defaults
  to `background120`, clamped to `UIScreen.maximumFramesPerSecond`, so ProMotion
  devices run at 120 and everything else at 60. Before, the plist said 60 and the
  registered defaults didn't mention the key at all, so the default was always 60.

Simulator result: the display went from about 30 distinct images/s (137–159 per
5 s, label ~50 FPS) to every guest frame presented at 60 Hz. p95 went 20 → 19 ms.

### Drawable and GL
- `window.rs`: an RGB888 drawable with no depth or stencil and retained backing off.
  SDL's UIKit view otherwise picks RGB565 (dithered banding) and allocates a depth
  buffer the compositor never uses.
- `gles_guest.rs`: the fog-state clamp is skipped on Apple GL, which accepts the
  values natively.
- `composition.rs`: an unchanged composite is skipped. `composite_key` captures
  every input the compositor reads (frame generation, viewport, rotation, cursor,
  and per-layer geometry, opacity, contents and textures). Animated layers always
  composite. At 120 Hz with a 60 fps guest this halves composites and presents.
  `MINECRAFT_COMPOSITE_SKIP=0` turns it off.
- `animation.rs`: presentation layers no longer clone the full-frame
  `presented_pixels` buffer on every composite.
- `composition.rs`/`ca_layer.rs`: the per-upload CPU pass that forced alpha to 255
  is replaced by the opaque texenv combiner (`presented_pixels_opaque`). In
  `gles/direct_texture.rs`, `opaque_begin`/`opaque_end` save the RGB/alpha scales as
  floats so the GL2 backend works too.
- `direct_texture.rs`: binding the owner's main FBO skips three attachment queries.
  The error drain is kept.
- `gles_guest.rs`: `glGetError` logs each code at most 16 times.
- `minecraft_presentation.rs`: `orient` copies rows directly for 0°/180° and does a
  32×32 tiled transpose for quarter turns (simulator CA presenter path).

### Interpreter
- `rust-entry/Cargo.toml`: release builds use `codegen-units = 1` and fat LTO, so
  guest memory access inlines into the dispatch loop.
- `mem.rs`: `Mem::read_fallible`/`write_fallible` replace slice-building
  `get_bytes_fallible` on every fetch, load and store (−13% ns/insn on straight-line
  code, −8% mixed, host benchmark).
- `in_it_block` is a single CPSR mask test.
- Leaf host calls: `___udivsi3`/`___umodsi3`/`___divsi3`/`___modsi3` reached through
  lazily-bound stubs finish inside `step_one` when the divisor is non-zero, skipping
  the SVC round trip (about 20% per division). They're disabled when
  `zero_stack_after_guest_to_host_call` is set.
- Tried and reverted after ablation (neutral or slower): LDM/STM/VLDM fast paths, a
  condition lookup table, and a combined 4-byte Thumb fetch.

Simulator, same world, with everything in: ~197 → ~216 MIPS, and p95 frame time
19.0 → 18.8 ms (HEAD was ~20 ms).

### Native kernels
- `minecraft_worldgen.rs`: the native dispatch chain moved into a cold
  `minecraft_native` match, which keeps the per-instruction loop small. Neutral
  within noise on the host.
- `Scripts/generate-loading-blocks.py` + regenerated `minecraft_loading.rs`: new
  islands for the rest of `Tessellator::vertex` and `vector<u8>::__append`, chained
  so a vertex stays native from entry to exit (about 2.4× faster per vertex, host
  benchmark). They're shadow-verified like the existing islands, and
  `MINECRAFT_NATIVE_LOADING=0` turns all loading islands off.
- A `minecraft_native_window` diagnostics event (profile mode only) reports
  per-block calls and retired instructions.

A note for whoever chases visuals next: on the simulator the title-screen panorama
sometimes shows a bright translucent wedge from the top-left corner, or faint
straight polygon edges. Frame-by-frame screen recordings show it on clean HEAD as
well, so it isn't caused by this work. Expect it to be how the software renderer
draws the panorama or its fog. Check on device before treating it as a bug.

### libc and runtime
- `posix_io.rs`/`fs.rs`: `close()` no longer calls `sync_all` (F_FULLFSYNC), which
  caused save hitches. `fsync` is a plain fsync. `fcntl(F_FULLFSYNC)` is implemented
  and still does the full flush.
- `errno.rs`, `pthread/key.rs`: per-thread storage is a `Vec` indexed by `ThreadId`
  instead of a SipHash map.
- `fx_hash.rs`: an in-crate Fx hasher for the ObjC object, method, mutex and condvar
  maps. None of these maps is iterated in an order-sensitive way.
- `mem/allocator.rs`: `free()` and heap growth merge with free neighbours on both
  sides, which reduces fragmentation and bucket churn.
- `stdlib.rs`: `free(NULL)` returns before the ObjC object lookup.
- `log.rs`: each line is formatted once and written with a single `write_all`.

### Guest-visible changes
- **Zero divisor** in compiler-rt division returns 0 and leaves `*remainder`
  untouched, as the armv7s hardware-divide path does on device. It used to abort the
  emulator.
- **`nearbyintf`** is implemented. It used to panic as unimplemented.
- **Heap addresses** can differ after some frees and grows because of the two-sided
  merge. Both behaviours are valid malloc.
- **Durability:** `close()` without `fsync` no longer forces data to disk, matching
  POSIX and iOS. An app kill doesn't lose page-cache data; only a kernel panic or
  power loss could.

## Remaining work, in order of expected payoff

1. **Device verification at 120 Hz.** Run with `MINECRAFT_PROFILE=1` on a ProMotion
   iPhone/iPad (TerrainProcessing now defaults to `background120`). Check that
   `native_refresh_window.callbacks` is ~600/5 s and how many of those carry new
   images, and repeat the interpreter A/B with the LTO release build. Watch thermals
   over a long session; if it throttles, `background60` is one Settings toggle away.
2. **Block decode cache.** `step_one` decodes every instruction from scratch. A
   per-basic-block cache is the next big interpreter win. It needs invalidation on
   dyld stub rewrites and on the code patches applied at load time
   (`invalidate_cache_range` is the hook).
3. **Weighted worker scheduling.** Every guest thread shares one host thread with a
   100k-instruction round-robin quantum. Giving the render thread priority over the
   terrain worker raises fps but slows chunk loading, which is a product tradeoff.
4. **Tessellator follow-ups.** The realloc path of `vector<u8>::__append` still
   leaves the islands for interpreted libc++ (operator new and memcpy).
   `MINECRAFT_WORLDGEN_SHADOW_ONLY=1` shadow-checks every island call, which is the
   way to validate any new island on device.
5. **Phase jitter at 60 Hz.** With the honest counter, about 3 guest frames per
   second land two to a refresh interval and one is never shown (this happened
   before too; the old counter hid it). Presenting on the link's `targetTimestamp`
   instead of on arrival would fix it.

Removed during this pass: a worker "idle yield" hook that ended the terrain worker's
quantum when its polling dequeue came back empty. Profiling showed MCPE runs that
worker in its blocking mode (`condition_variable` timed wait), so the hook never fired.

## Known pre-existing issues

- `cpu::interpreter::minecraft_worldgen::tests::minecraft_probe_noise_differential`
  fails on clean HEAD ("FP case 904"). It isn't caused by this work.
- Several Environment-backed tests race when run in parallel. Use
  `cargo test --lib --release -- --test-threads=1`.
