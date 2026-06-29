## ADDED Requirements

### Requirement: Non-child webview set_bounds is cache-only (correct behavior)
`set_bounds()` for **non-child (main) webviews** SHALL be cache-only: it updates `bounds_cache` (returned by `bounds()`) but does NOT call ArkTS `setBounds`. This is because the main webview's Web component uses `data.style.width/height = "100%"` to fill the window; calling ArkTS `setBounds` would replace `"100%"` with specific pixel values, causing black bars on fullscreen/resize. Child webviews SHALL continue calling ArkTS `setBounds` (they use absolute positioning).

#### Scenario: non-child set_bounds is cache-only
- **WHEN** `set_bounds(rect)` is called on a non-child (main) webview
- **THEN** the system updates `bounds_cache` only (no ArkTS call), returns `Ok(())`, and the Web component continues filling the window via `"100%"`

#### Scenario: child set_bounds calls ArkTS setBounds
- **WHEN** `set_bounds(rect)` is called on a child webview
- **THEN** the system calls ArkTS `setBounds(x, y, w, h)` + updates `bounds_cache`

#### Scenario: fullscreen no black bars
- **WHEN** the app window is maximized/fullscreen
- **THEN** the Web content fills the entire window with no black bars on any side (cache-only ensures `"100%"` is preserved)

## MODIFIED Requirements

### Requirement: Transparent background verified closed
The transparent background support (archive `p1-webview-transparent`) is verified as fully implemented: `ArkHelper.ets` sets `init.transparent=true` when the `transparent` flag is set; `DefaultWebview.ets` uses `RenderMode.SYNC_RENDER` for transparent webviews; `DefaultXComponent.ets` has defensive transparent containers; `set_background_color` dynamically updates via monkey-patch. No code change needed — R74 is closed.

#### Scenario: transparent webview renders with transparent background
- **WHEN** a webview is created with `transparent: true`
- **THEN** the Web component uses `RenderMode.SYNC_RENDER` and the container is transparent (verified by archive `p1-webview-transparent`, no change in this phase)
