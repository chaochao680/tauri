<script>
  import { onMount } from 'svelte';
  import { runTests } from '../lib/test-runner';
  import { coreTests } from '../lib/tests/core';
  import { pluginTests } from '../lib/tests/plugins';
  import { dpiTests } from '../lib/tests/dpi';
  import { windowDpiTests } from '../lib/tests/window-dpi';
  import { imageTests } from '../lib/tests/image';
  import { menuTests } from '../lib/tests/menu';
  import { trayTests } from '../lib/tests/tray';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow, currentMonitor } from '@tauri-apps/api/window';
  import { appCacheDir } from '@tauri-apps/api/path';
  import { flushConsoleLog, clearConsoleLog } from '../lib/console-capture';

  let { onMessage } = $props();

  let results = $state([]);
  let running = $state(false);
  let report = $state(null);

  // Manual test state
  let manualResult = $state('');
  let focusWatchActive = $state(false);
  let focusWatchUnlisten = null;
  let focusEvents = $state([]);

  const allTests = [...coreTests, ...pluginTests, ...dpiTests, ...windowDpiTests, ...imageTests, ...menuTests, ...trayTests];

  async function runAll() {
    running = true;
    results = [];
    report = null;
    onMessage('--- Test Run Started ---');

    // Clear previous test report before starting
    try {
      await invoke('clear_test_report');
    } catch (e) {
      onMessage(`Failed to clear report: ${e}`);
    }

    // Skip manual tests - they require user interaction
    const filtered = allTests.filter((t) => t.category !== 'manual');

    const r = await runTests(filtered, (result, index, total) => {
      results = [...results, result];
      const icon = result.status === 'pass' ? '[PASS]' : result.status === 'fail' ? '[FAIL]' : '[SKIP]';
      const msg = `${icon} ${result.name}${result.error ? ' - ' + result.error : ''} (${result.duration}ms)`;
      onMessage(msg);
    });

    report = r;
    onMessage(`--- Done: ${r.passed} passed, ${r.failed} failed, ${r.skipped} skipped ---`);
    running = false;
  }

  // Auto-run on first mount
  onMount(() => {
    runAll();
  });

  async function runCategory(category) {
    running = true;
    results = [];
    report = null;
    const filtered = allTests.filter((t) => t.category === category);
    onMessage(`--- Running ${category} tests (${filtered.length}) ---`);

    const r = await runTests(filtered, (result) => {
      results = [...results, result];
      const icon = result.status === 'pass' ? '[PASS]' : result.status === 'fail' ? '[FAIL]' : '[SKIP]';
      onMessage(`${icon} ${result.name}${result.error ? ' - ' + result.error : ''}`);
    });

    report = r;
    onMessage(`--- Done: ${r.passed} passed, ${r.failed} failed, ${r.skipped} skipped ---`);
    running = false;
  }

  async function wrapManual(name, fn) {
    const start = Date.now();
    console.log('[ManualTest] Starting:', name);
    try {
      await fn();
      if (manualResult) {
        console.log('[ManualTest]', manualResult);
      }
      console.log('[ManualTest] Completed:', name, 'in', Date.now() - start, 'ms');
    } catch (e) {
      console.error('[ManualTest] Failed:', name, e);
    }
    try {
      const path = await flushConsoleLog();
      onMessage(`Console log saved: ${path}`);
    } catch (e) {
      onMessage(`Failed to save console log: ${e}`);
    }
  }

  // ─── Manual Tests ───
  async function manualIsFocused() {
    await wrapManual('isFocused', async () => {
      const focused = await getCurrentWindow().isFocused();
      const ok = focused === true;
      manualResult = `isFocused() → ${focused} ${ok ? '[OK: app in foreground]' : '[UNEXPECTED: should be true since you clicked the button]'}`;
      onMessage(manualResult);
    });
  }

  async function toggleFocusWatch() {
    if (focusWatchActive) {
      focusWatchUnlisten?.();
      focusWatchUnlisten = null;
      focusWatchActive = false;
      manualResult = `Stopped watching focus changes. Total events: ${focusEvents.length}`;
      onMessage(manualResult);
    } else {
      focusEvents = [];
      focusWatchUnlisten = await getCurrentWindow().onFocusChanged(({ payload }) => {
        const ts = new Date().toLocaleTimeString();
        focusEvents = [...focusEvents, `${ts}: focused=${payload}`];
        onMessage(`[onFocusChanged] focused=${payload}`);
      });
      focusWatchActive = true;
      manualResult = 'Watching focus changes. Send the app to background and back to trigger events.';
      onMessage(manualResult);
    }
    try {
      const path = await flushConsoleLog();
      onMessage(`Console log saved: ${path}`);
    } catch (e) {}
  }

  async function manualMonitor() {
    await wrapManual('currentMonitor', async () => {
      const m = await currentMonitor();
      if (!m) {
        manualResult = 'currentMonitor() → null';
      } else {
        manualResult = `Monitor: ${m.size.width}×${m.size.height} @ scale ${m.scaleFactor} | position (${m.position.x}, ${m.position.y}) | name "${m.name ?? ''}"`;
      }
      onMessage(manualResult);
    });
  }

  async function manualAppCacheDir() {
    await wrapManual('appCacheDir', async () => {
      const dir = await appCacheDir();
      manualResult = `appCacheDir() → ${dir}`;
      onMessage(manualResult);
    });
  }

  async function manualWindowDpi() {
    await wrapManual('windowDpi', async () => {
      const win = getCurrentWindow();
      const inner = await win.innerSize();
      const outer = await win.outerSize();
      const innerPos = await win.innerPosition();
      const outerPos = await win.outerPosition();
      const scale = await win.scaleFactor();

      manualResult = `innerSize: ${inner.width}×${inner.height}
outerSize: ${outer.width}×${outer.height}
innerPosition: (${innerPos.x}, ${innerPos.y})
outerPosition: (${outerPos.x}, ${outerPos.y})
scaleFactor: ${scale}

Expected behavior:
• Resize window → innerSize/outerSize should change
• Drag window → positions should change
• outerSize >= innerSize (includes window decorations)
• scaleFactor typically 1.0-3.0 (depends on display DPI)`;
      onMessage(manualResult);
    });
  }

  // ─── Menu Bar Manual Tests ───
  const MB_TEST_ICON = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';

  async function manualMenuBarRestore() {
    await wrapManual('menuBarRestore', async () => {
      const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
      const fileSub = await Submenu.new({ text: 'File', items: [
        await PredefinedMenuItem.new({ item: 'CloseWindow' }),
        await PredefinedMenuItem.new({ item: 'Quit' }),
      ]});
      const editSub = await Submenu.new({ text: 'Edit', items: [
        await PredefinedMenuItem.new({ item: 'Undo' }),
        await PredefinedMenuItem.new({ item: 'Redo' }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await PredefinedMenuItem.new({ item: 'Cut' }),
        await PredefinedMenuItem.new({ item: 'Copy' }),
        await PredefinedMenuItem.new({ item: 'Paste' }),
        await PredefinedMenuItem.new({ item: 'SelectAll' }),
      ]});
      const windowSub = await Submenu.new({ text: 'Window', items: [
        await PredefinedMenuItem.new({ item: 'Minimize' }),
        await PredefinedMenuItem.new({ item: 'Maximize' }),
        await PredefinedMenuItem.new({ item: 'CloseWindow' }),
      ]});
      const helpSub = await Submenu.new({ text: 'Help', items: [
        await PredefinedMenuItem.new({ item: { About: { name: 'Tauri API Validation' } } }),
      ]});
      const menu = await Menu.new({ items: [fileSub, editSub, windowSub, helpSub] });
      await menu.setAsWindowMenu();
      manualResult = 'Default menu restored: File | Edit | Window | Help';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarVisible() {
    await wrapManual('menuBarVisible', async () => {
      const visible = await invoke('plugin:app-menu|is_menu_visible');
      manualResult = `is_menu_visible() = ${visible}\nCheck: Top of window should show a menu bar with submenu labels.\nIf visible and ${visible} === true → PASS.\nTip: Click "Restore Default Menu" first if menu bar is missing.`;
      onMessage(manualResult);
    });
  }

  async function manualMenuBarDropdown() {
    await wrapManual('menuBarDropdown', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const sub = await Submenu.new({ text: 'Click Me', items: [
        await MenuItem.new({ text: 'Item A' }),
        await MenuItem.new({ text: 'Item B' }),
      ]});
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "Click Me" submenu.\nClick "Click Me" → dropdown should appear with "Item A" and "Item B".\nIf dropdown appears → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarNested() {
    await wrapManual('menuBarNested', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const inner = await Submenu.new({ text: 'Inner', items: [
        await MenuItem.new({ text: 'Deep Item' }),
      ]});
      const outer = await Submenu.new({ text: 'Outer', items: [
        await MenuItem.new({ text: 'Top Item' }),
        inner,
      ]});
      const menu = await Menu.new({ items: [outer] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "Outer → Top Item + Inner → Deep Item".\nClick Outer → hover Inner → should show nested dropdown with "Deep Item".\nIf nested submenu works → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarHover() {
    await wrapManual('menuBarHover', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const sub = await Submenu.new({ text: 'HoverTest', items: [
        await MenuItem.new({ text: 'Item' }),
      ]});
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "HoverTest".\nHover mouse over "HoverTest" → background should change color.\nMove away → background returns to normal.\nIf hover effect visible → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarDarkMode() {
    await wrapManual('menuBarDarkMode', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const sub = await Submenu.new({ text: 'DarkTest', items: [
        await MenuItem.new({ text: 'Item' }),
      ]});
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Action: Switch system to dark mode (Settings → Display → Dark).\nCheck: Menu bar background → dark, text → light.\nSwitch back → background → light, text → dark.\nIf colors adapt → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarBarIcon() {
    await wrapManual('menuBarBarIcon', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const sub = await Submenu.new({ text: 'IconMenu', icon: MB_TEST_ICON, items: [
        await MenuItem.new({ text: 'Item' }),
      ]});
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "IconMenu" submenu WITH icon.\nBar-level "IconMenu" should show a small icon next to the text.\nIf icon visible at bar level → PASS. If only text → FAIL.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarDisabledItem() {
    await wrapManual('menuBarDisabledItem', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const sub = await Submenu.new({ text: 'DisTest', items: [
        await MenuItem.new({ text: 'Disabled', enabled: false }),
        await MenuItem.new({ text: 'Normal', enabled: true }),
      ]});
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "DisTest → Disabled + Normal".\nClick DisTest → "Disabled" should appear grayed out + semi-transparent.\n"Normal" should appear with full color.\nIf disabled visual correct → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarHide() {
    await wrapManual('menuBarHide', async () => {
      await invoke('plugin:app-menu|hide_menu');
      const visible = await invoke('plugin:app-menu|is_menu_visible');
      manualResult = `hide_menu() called. is_menu_visible() = ${visible}\nCheck: Menu bar should disappear from top of window.\nIf disappeared and ${visible} === false → PASS.\nClick "Show" button to restore.`;
      onMessage(manualResult);
    });
  }

  async function manualMenuBarShow() {
    await wrapManual('menuBarShow', async () => {
      await manualMenuBarRestore();
      await invoke('plugin:app-menu|show_menu');
      const visible = await invoke('plugin:app-menu|is_menu_visible');
      manualResult = `show_menu() called (default menu restored). is_menu_visible() = ${visible}\nCheck: Menu bar should reappear at top of window.\nIf visible and ${visible} === true → PASS.`;
      onMessage(manualResult);
    });
  }

  async function manualMenuBarIsMenuVisible() {
    await wrapManual('menuBarIsMenuVisible', async () => {
      const visible = await invoke('plugin:app-menu|is_menu_visible');
      manualResult = `is_menu_visible() = ${visible}\nExpected: true (menu bar visible by default).\nIf true → PASS.\nTip: Click "Hide" first, then click this button → should return false.`;
      onMessage(manualResult);
    });
  }

  async function manualMenuBarRemove() {
    await wrapManual('menuBarRemove', async () => {
      const { Menu } = await import('@tauri-apps/api/menu');
      const emptyMenu = await Menu.new({ items: [] });
      await emptyMenu.setAsWindowMenu();
      manualResult = 'Empty menu set as window menu (remove_menu equivalent).\nCheck: Menu bar should disappear (no items to show).\nIf disappeared → PASS.\nClick "Restore Default Menu" to restore.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarAutoRefreshText() {
    await wrapManual('menuBarAutoRefreshText', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const item = await MenuItem.new({ text: 'Original' });
      const sub = await Submenu.new({ text: 'Refresh', items: [item] });
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      await new Promise(r => setTimeout(r, 500));
      await item.setText('Updated!');
      manualResult = 'Menu bar: "Refresh → Original".\nsetText("Updated!") called → auto_refresh should push update.\nClick "Refresh" dropdown → should show "Updated!" (not "Original").\nIf text updated → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarAutoRefreshChecked() {
    await wrapManual('menuBarAutoRefreshChecked', async () => {
      const { Menu, Submenu, CheckMenuItem } = await import('@tauri-apps/api/menu');
      const check = await CheckMenuItem.new({ text: 'Check Me', checked: false });
      const sub = await Submenu.new({ text: 'Refresh', items: [check] });
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      await new Promise(r => setTimeout(r, 500));
      await check.setChecked(true);
      manualResult = 'Menu bar: "Refresh → Check Me" (unchecked).\nsetChecked(true) called → auto_refresh should update.\nClick "Refresh" dropdown → "Check Me" should show a checkmark.\nIf checked state updated → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarAccelerator() {
    await wrapManual('menuBarAccelerator', async () => {
      const { Menu, Submenu, MenuItem } = await import('@tauri-apps/api/menu');
      const item = await MenuItem.new({
        text: 'Accel Test',
        action: (id) => {
          console.log('[MenuBarTest] Accelerator fired! id:', id);
          manualResult = `Accelerator Ctrl+O FIRED! id=${id}`;
          onMessage(manualResult);
        }
      });
      await item.setAccelerator('Ctrl+O');
      const sub = await Submenu.new({ text: 'Accel', items: [item] });
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "Accel → Accel Test" (Ctrl+O).\nPress Ctrl+O → should trigger action callback → show "FIRED" message.\nAlso try clicking "Accel Test" in dropdown → should also fire.\nIf Ctrl+O triggers → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarAcceleratorCopy() {
    await wrapManual('menuBarAcceleratorCopy', async () => {
      const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
      const copyItem = await PredefinedMenuItem.new({ item: 'Copy' });
      const sub = await Submenu.new({ text: 'Edit', items: [copyItem] });
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "Edit → Copy" (Ctrl+C built-in).\nType some text → select it → press Ctrl+C.\nThen try pasting → should paste the copied text.\nIf Ctrl+C copies → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarFullscreen() {
    await wrapManual('menuBarFullscreen', async () => {
      const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
      const fsItem = await PredefinedMenuItem.new({ item: 'Fullscreen' });
      const sub = await Submenu.new({ text: 'View', items: [fsItem] });
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "View → Fullscreen".\nClick "View → Fullscreen" → window enters fullscreen, menu bar should disappear.\nPress Esc or click again → exit fullscreen, menu bar should recover.\nIf menu bar hides/recover → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarPredefinedHide() {
    await wrapManual('menuBarPredefinedHide', async () => {
      const { Menu, Submenu, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
      const hideItem = await PredefinedMenuItem.new({ item: 'Hide' });
      const sub = await Submenu.new({ text: 'Window', items: [hideItem] });
      const menu = await Menu.new({ items: [sub] });
      await menu.setAsWindowMenu();
      manualResult = 'Menu bar: "Window → Hide".\nClick "Window → Hide" → window should minimize.\nRestore window from taskbar → confirm it reappears.\nIf window minimizes on Hide → PASS.';
      onMessage(manualResult);
    });
  }

  async function manualMenuBarPopupRegression() {
    await wrapManual('menuBarPopupRegression', async () => {
      const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
      const item = await MenuItem.new({ text: 'Popup Test' });
      const menu = await Menu.new({ items: [item] });
      await menu.popup();
      manualResult = 'Popup menu triggered at cursor position.\nCheck: Context menu should appear with "Popup Test".\nThis verifies AppStorage key renaming did not break popup.\nIf popup appears → PASS.';
      onMessage(manualResult);
    });
  }

  // ─── Tray Manual Tests ───
  async function manualTrayIconShow() {
    await wrapManual('trayIconShow', async () => {
      const { TrayIcon } = await import('@tauri-apps/api/tray');
      const TEST_ICON = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAAJUlEQVR4nGNImfb/Py0xw6gFoxaMWjBqwagFoxaMWjBqwdCwAAB3Wq5b2Gx59gAAAABJRU5ErkJggg==';
      console.log('[Manual Tray] Creating tray icon...');
      const tray = await TrayIcon.new({ icon: TEST_ICON, tooltip: 'Test Tray Icon' });
      console.log(`[Manual Tray] Tray created with id: ${tray.id}`);
      manualResult = `Tray icon created with id: "${tray.id}".\nCheck the status bar (bottom of screen) for a blue square icon.`;
      onMessage(manualResult);
    });
  }

  async function manualTrayEvent() {
    await wrapManual('trayEvent', async () => {
      const { TrayIcon } = await import('@tauri-apps/api/tray');
      const TEST_ICON = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAAJUlEQVR4nGNImfb/Py0xw6gFoxaMWjBqwagFoxaMWjBqwdCwAAB3Wq5b2Gx59gAAAABJRU5ErkJggg==';
      console.log('[Manual Tray] Creating tray with event listener...');
      const tray = await TrayIcon.new({
        icon: TEST_ICON,
        tooltip: 'Click me!',
        action: (event) => {
          const data = JSON.stringify(event);
          console.log(`[Manual Tray] Event received: ${data}`);
          manualResult = `Tray event received:\n${data}`;
          onMessage(manualResult);
        }
      });
      console.log(`[Manual Tray] Tray created with id: ${tray.id}`);
      manualResult = `Tray created with id: "${tray.id}".\nClick the status bar icon to trigger events.\nResult will appear below.`;
      onMessage(manualResult);
    });
  }

  async function manualTrayMenu() {
    await wrapManual('trayMenu', async () => {
      const { TrayIcon } = await import('@tauri-apps/api/tray');
      const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
      const TEST_ICON = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAAJUlEQVR4nGNImfb/Py0xw6gFoxaMWjBqwagFoxaMWjBqwdCwAAB3Wq5b2Gx59gAAAABJRU5ErkJggg==';
      console.log('[Manual Tray] Creating tray with menu...');
      const item = await MenuItem.new({ text: 'Test Menu Item' });
      console.log('[Manual Tray] Menu item created');
      const menu = await Menu.new({ items: [item] });
      console.log('[Manual Tray] Menu created');
      const tray = await TrayIcon.new({ icon: TEST_ICON, menu, tooltip: 'Right-click me' });
      console.log(`[Manual Tray] Tray created with id: ${tray.id}`);
      manualResult = `Tray created with menu.\nRight-click the status bar icon to see the context menu.\nClick the menu item to verify event trigger.`;
      onMessage(manualResult);
    });
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex gap-2 flex-wrap">
    <button class="btn" onclick={runAll} disabled={running}>
      {running ? 'Running...' : 'Run All'}
    </button>
    <button class="btn" onclick={() => runCategory('auto')} disabled={running}>
      Run Auto
    </button>
    <button class="btn" onclick={() => runCategory('side-effect')} disabled={running}>
      Run Side-Effect
    </button>
    <button class="btn" onclick={async () => {
      try {
        await clearConsoleLog();
        onMessage('Console log cleared');
      } catch (e) {
        onMessage(`Failed to clear: ${e}`);
      }
    }}>
      Clear Console
    </button>
  </div>

  {#if report}
    <div class="text-sm mt-2 p-2 rd-1 bg-black/10 dark:bg-white/10">
      Total: {report.total} | Passed: {report.passed} | Failed: {report.failed} | Skipped: {report.skipped}
    </div>
  {/if}

  {#if results.length > 0}
    <div class="flex flex-col gap-1 mt-2 text-xs max-h-60 overflow-y-auto">
      {#each results as r}
        <div class="flex items-center gap-2 p-1 rd-1 {r.status === 'pass' ? 'bg-green-500/10' : r.status === 'fail' ? 'bg-red-500/10' : 'bg-gray-500/10'}">
          <span class="font-mono w-12 shrink-0">
            {r.status === 'pass' ? 'PASS' : r.status === 'fail' ? 'FAIL' : 'SKIP'}
          </span>
          <span class="flex-1 truncate">{r.name}</span>
          <span class="text-gray-500 shrink-0">{r.duration}ms</span>
        </div>
      {/each}
    </div>
  {/if}

  <div class="mt-4 pt-3 border-t-1 border-solid border-code">
    <h4 class="my-2">Manual Tests</h4>
    <p class="text-xs text-gray-500 mb-2">
      Verifies behavior that autotest can't cover (e.g., focus state must be true when user is interacting).
    </p>
    <div class="flex gap-2 flex-wrap">
      <button class="btn" onclick={manualIsFocused}>isFocused (should be true)</button>
      <button class="btn" onclick={toggleFocusWatch}>
        {focusWatchActive ? 'Stop watching focus' : 'Watch onFocusChanged'}
      </button>
      <button class="btn" onclick={manualMonitor}>currentMonitor</button>
      <button class="btn" onclick={manualAppCacheDir}>appCacheDir</button>
      <button class="btn" onclick={manualWindowDpi}>Window DPI (resize/drag to verify)</button>
    </div>
    <div class="mt-2 pt-2 border-t-1 border-solid border-code">
      <h5 class="my-1 text-xs text-gray-500">Tray Manual Tests</h5>
      <div class="flex gap-2 flex-wrap">
        <button class="btn" onclick={manualTrayIconShow}>Tray Icon Show (check system tray)</button>
        <button class="btn" onclick={manualTrayEvent}>Tray Event (click icon to trigger)</button>
        <button class="btn" onclick={manualTrayMenu}>Tray Menu (right-click to see menu)</button>
      </div>
    </div>
    <div class="mt-2 pt-2 border-t-1 border-solid border-code">
      <h5 class="my-1 text-xs text-gray-500">Menu Bar Manual Tests</h5>
      <div class="flex gap-2 flex-wrap">
        <button class="btn" onclick={manualMenuBarRestore}>Restore Default Menu</button>
        <button class="btn" onclick={manualMenuBarVisible}>MenuBar Visible</button>
        <button class="btn" onclick={manualMenuBarDropdown}>MenuBar Dropdown</button>
        <button class="btn" onclick={manualMenuBarNested}>MenuBar Nested Submenu</button>
        <button class="btn" onclick={manualMenuBarHover}>MenuBar Hover</button>
        <button class="btn" onclick={manualMenuBarDarkMode}>MenuBar Dark Mode</button>
        <button class="btn" onclick={manualMenuBarBarIcon}>MenuBar Bar-Level Icon</button>
        <button class="btn" onclick={manualMenuBarDisabledItem}>MenuBar Disabled Item</button>
        <button class="btn" onclick={manualMenuBarHide}>MenuBar Hide</button>
        <button class="btn" onclick={manualMenuBarShow}>MenuBar Show</button>
        <button class="btn" onclick={manualMenuBarIsMenuVisible}>MenuBar is_menu_visible</button>
        <button class="btn" onclick={manualMenuBarRemove}>MenuBar Remove Menu</button>
        <button class="btn" onclick={manualMenuBarAutoRefreshText}>MenuBar Auto Refresh Text</button>
        <button class="btn" onclick={manualMenuBarAutoRefreshChecked}>MenuBar Auto Refresh Checked</button>
        <button class="btn" onclick={manualMenuBarAccelerator}>MenuBar Accelerator Ctrl+O</button>
        <button class="btn" onclick={manualMenuBarAcceleratorCopy}>MenuBar Accelerator Ctrl+C</button>
        <button class="btn" onclick={manualMenuBarFullscreen}>MenuBar Fullscreen</button>
        <button class="btn" onclick={manualMenuBarPredefinedHide}>MenuBar Predefined Hide</button>
        <button class="btn" onclick={manualMenuBarPopupRegression}>MenuBar Popup Regression</button>
      </div>
    </div>
    {#if manualResult}
      <div class="mt-2 p-2 rd-1 bg-black/10 dark:bg-white/10 text-xs font-mono break-all">
        {manualResult}
      </div>
    {/if}
    {#if focusWatchActive || focusEvents.length > 0}
      <div class="mt-2 text-xs">
        <div class="font-bold mb-1">Focus events ({focusEvents.length}):</div>
        <div class="max-h-32 overflow-y-auto flex flex-col gap-1">
          {#each focusEvents as ev}
            <div class="font-mono p-1 rd-1 bg-black/5 dark:bg-white/5">{ev}</div>
          {/each}
          {#if focusEvents.length === 0 && focusWatchActive}
            <div class="text-gray-500 italic">Waiting... send app to background and bring it back.</div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
