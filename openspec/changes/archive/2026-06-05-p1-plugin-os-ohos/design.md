## Context

Tauri's OS plugin (`plugins-workspace/plugins/os/`) provides platform detection at compile time. Values are baked into a JavaScript init script via `InitJavascript::new()` and exposed to frontend code through `window.__TAURI_OS_PLUGIN_INTERNALS__`.

On OHOS, the Rust target triple is `aarch64-unknown-linux-ohos`:
- `target_os = "linux"` → `std::env::consts::OS` returns `"linux"`
- `target_env = "ohos"` → the **only** compile-time way to detect OHOS
- `std::env::consts::FAMILY` returns `"unix"`
- `std::env::consts::ARCH` returns `"aarch64"`

The `os_info` crate does not support OHOS and would return Linux distribution info (or fail).

### Current State

```rust
// lib.rs — returns "linux" on OHOS
pub fn platform() -> &'static str {
    std::env::consts::OS
}

// lib.rs — returns OsType::Linux on OHOS (matches target_os = "linux" branch)
pub fn type_() -> OsType {
    #[cfg(any(target_os = "linux", ...))]
    return OsType::Linux;
    // ...
}
```

```typescript
// index.ts — no 'ohos' in type definitions
type Platform = 'linux' | 'macos' | 'ios' | ... | 'windows'
type OsType = 'linux' | 'windows' | 'macos' | 'ios' | 'android'
```

## Goals / Non-Goals

**Goals:**
- `platform()` SHALL return `"ohos"` on OHOS (both desktop and mobile)
- `type_()` SHALL return `OsType::Ohos` (displayed as `"ohos"`) on OHOS
- `version()` SHALL return a meaningful OHOS version string (not Linux distro info)
- TypeScript type definitions SHALL include `'ohos'` for `Platform` and `OsType`
- All changes SHALL be isolated via `cfg(target_env = "ohos")` — zero impact on other platforms

**Non-Goals:**
- Implementing OHOS-specific `hostname()` or `locale()` (current native APIs may already work via libc)
- Implementing OHOS-specific `family()` (returning `"unix"` is correct for OHOS)
- Implementing OHOS-specific `arch()` (returning `"aarch64"` is correct for OHOS)
- Runtime OHOS version detection via ArkTS/NAPI (this is a compile-time fix only)
- Modifying `os_info` crate to support OHOS

## Decisions

### Decision 1: `cfg(target_env = "ohos")` guard placement in `type_()`

**Choice**: Add `cfg(target_env = "ohos")` as the **first** branch in `type_()`, before the `cfg(target_os = "linux")` check.

**Rationale**: Since OHOS has `target_os = "linux"`, it matches the existing Linux branch. The OHOS check must come first to prevent fallthrough. This is consistent with how the tauri core handles OHOS (see `crates/tauri/Cargo.toml` where Linux deps use `not(target_env = "ohos")`).

**Alternative considered**: Modifying the Linux branch to `cfg(all(target_os = "linux", not(target_env = "ohos")))`. Rejected because adding a new first branch is cleaner and doesn't require touching existing code.

### Decision 2: `platform()` override strategy

**Choice**: Return a hardcoded `"ohos"` string when `cfg(target_env = "ohos")` is true.

```rust
pub fn platform() -> &'static str {
    #[cfg(target_env = "ohos")]
    return "ohos";
    #[cfg(not(target_env = "ohos"))]
    std::env::consts::OS
}
```

**Rationale**: `std::env::consts::OS` is a compile-time constant determined by the target triple. There is no way to change its value. A `cfg`-based override is the only viable approach. The return type is `&'static str`, which is compatible with string literals.

### Decision 3: `version()` on OHOS

**Choice**: Return `os_info::Version` with a hardcoded OHOS version string.

```rust
#[cfg(target_env = "ohos")]
pub fn version() -> Version {
    Version::Semantic(0, 0, 0)
}
```

**Rationale**: The `os_info::Version` type is used in the init JavaScript. The `os_info` crate doesn't support OHOS, so we must provide a fallback. Using `0.0.0` as a placeholder is safe — the init JavaScript calls `.to_string()` on it. If OHOS version detection is needed later, it can be added as a runtime command (similar to `locale` and `hostname`).

**Alternative considered**: Parsing `/etc/openharmony-release` or similar system files. Rejected — this adds complexity and file I/O for a compile-time value. Can be added later if needed.

### Decision 4: TypeScript type extension

**Choice**: Add `'ohos'` to the existing `Platform` and `OsType` union types.

**Rationale**: Minimal, backward-compatible change. Frontend code that already handles `'linux'` won't break — it just won't match `'ohos'` (which is the correct behavior, since OHOS is not Linux).

## Risks / Trade-offs

- **[Risk] Frontend code hardcodes `'linux'` checks** → Frontend apps that use `if (platform() === 'linux')` will no longer match on OHOS. This is the **intended** behavior — OHOS should be treated distinctly from Linux. Apps need to add `'ohos'` to their platform checks if they want to treat it like Linux.

- **[Risk] `version()` returns `0.0.0`** → Consumers of `version()` will see a placeholder on OHOS. This is acceptable for now — the value is documented as platform-specific and the placeholder is clearly not a real version. Runtime version detection can be added later.

- **[Trade-off] Compile-time only** → This fix is purely compile-time. Runtime OHOS version detection (via ArkTS NAPI) would require adding a command handler, which is out of scope. The priority is fixing the incorrect "linux" return value.
