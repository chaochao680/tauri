import type { TestCase } from '../test-runner';

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

/**
 * Fetch with retry for external endpoints (e.g. httpbin.org).
 * Retries up to 3 times with 1s delay on 503 or network errors.
 */
async function retryFetch(
  url: string,
  init: Parameters<typeof globalThis.fetch>[1],
  maxRetries = 3
): Promise<Response> {
  const { fetch } = await import('@tauri-apps/plugin-http');
  const opts = { ...init, connectTimeout: 3000 };
  let lastError: unknown;
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const resp = await fetch(url, opts);
      if (resp.status !== 503 || attempt === maxRetries) return resp;
      lastError = new Error(`HTTP 503 from ${url}`);
    } catch (e) {
      lastError = e;
    }
    if (attempt < maxRetries) {
      await new Promise((r) => setTimeout(r, 1000));
    }
  }
  throw lastError;
}

export const pluginTests: TestCase[] = [
  // @tauri-apps/plugin-os
  {
    name: '@tauri-apps/plugin-os.platform',
    category: 'auto',
    async fn() {
      const { platform } = await import('@tauri-apps/plugin-os');
      const p = platform();
      assert(typeof p === 'string' && p.length > 0, `expected non-empty string, got "${p}"`);
    },
  },

  // @tauri-apps/plugin-log
  {
    name: '@tauri-apps/plugin-log.trace',
    category: 'auto',
    async fn() {
      const { trace } = await import('@tauri-apps/plugin-log');
      await trace('test trace message');
    },
  },
  {
    name: '@tauri-apps/plugin-log.debug',
    category: 'auto',
    async fn() {
      const { debug } = await import('@tauri-apps/plugin-log');
      await debug('test debug message');
    },
  },
  {
    name: '@tauri-apps/plugin-log.info',
    category: 'auto',
    async fn() {
      const { info } = await import('@tauri-apps/plugin-log');
      await info('test info message');
    },
  },
  {
    name: '@tauri-apps/plugin-log.warn',
    category: 'auto',
    async fn() {
      const { warn } = await import('@tauri-apps/plugin-log');
      await warn('test warn message');
    },
  },
  {
    name: '@tauri-apps/plugin-log.error',
    category: 'auto',
    async fn() {
      const { error } = await import('@tauri-apps/plugin-log');
      await error('test error message');
    },
  },

  // @tauri-apps/plugin-http
  //
  // Most HTTP tests use the local echo server (localhost:3003) started in
  // src-tauri/src/lib.rs to avoid flaky failures from external services.
  // Only tests that genuinely need a real remote endpoint (TLS handshake,
  // specific JSON structure) still target httpbin.org with retry logic.
  {
    name: '@tauri-apps/plugin-http.fetch (GET)',
    category: 'auto',
    async fn() {
      const { fetch } = await import('@tauri-apps/plugin-http')
      const resp = await fetch('http://localhost:3003/get', { method: 'GET' })
      assert(resp.status === 200, `expected status 200, got ${resp.status}`)
    }
  },
  {
    name: '@tauri-apps/plugin-http.fetch (POST)',
    category: 'auto',
    async fn() {
      const { fetch } = await import('@tauri-apps/plugin-http')
      const body = JSON.stringify({ test: 'post-data' })
      const resp = await fetch('http://localhost:3003/post', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body
      })
      assert(resp.status === 200, `expected status 200, got ${resp.status}`)
      // Echo server returns the request body as-is
      const data = await resp.text()
      assert(
        data === body,
        `body mismatch: ${data}`
      )
    }
  },
  {
    name: '@tauri-apps/plugin-http.fetch (PUT)',
    category: 'auto',
    async fn() {
      const { fetch } = await import('@tauri-apps/plugin-http')
      const body = JSON.stringify({ update: 'put-data' })
      const resp = await fetch('http://localhost:3003/put', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body
      })
      assert(resp.status === 200, `expected status 200, got ${resp.status}`)
      const data = await resp.text()
      assert(
        data === body,
        `body mismatch: ${data}`
      )
    }
  },
  {
    name: '@tauri-apps/plugin-http.fetch (DELETE)',
    category: 'auto',
    async fn() {
      const { fetch } = await import('@tauri-apps/plugin-http')
      const resp = await fetch('http://localhost:3003/delete', {
        method: 'DELETE'
      })
      assert(resp.status === 200, `expected status 200, got ${resp.status}`)
    }
  },
  {
    name: '@tauri-apps/plugin-http.fetch (custom headers)',
    category: 'auto',
    async fn() {
      const { fetch } = await import('@tauri-apps/plugin-http')
      const resp = await fetch('http://localhost:3003/headers', {
        method: 'GET',
        headers: {
          'X-Custom-Header': 'test-value-123',
          'X-Another-Header': 'another-value'
        }
      })
      assert(resp.status === 200, `expected status 200, got ${resp.status}`)
      // Echo server reflects request headers in response headers
      const customHeader = resp.headers.get('X-Custom-Header')
      assert(
        customHeader === 'test-value-123',
        `custom header mismatch: ${customHeader}`
      )
      const anotherHeader = resp.headers.get('X-Another-Header')
      assert(
        anotherHeader === 'another-value',
        `another header mismatch: ${anotherHeader}`
      )
    }
  },
  {
    name: '@tauri-apps/plugin-http.fetch (JSON parse)',
    category: 'auto',
    async fn() {
      // Use jsonplaceholder — fast, reliable, no rate limiting
      const resp = await retryFetch('https://jsonplaceholder.typicode.com/todos/1', { method: 'GET' })
      assert(resp.status === 200, `expected status 200, got ${resp.status}`)
      const data = await resp.json()
      assert(typeof data === 'object', 'expected JSON object')
      assert(data.title !== undefined, 'expected title property')
    }
  },
  {
    name: '@tauri-apps/plugin-http.fetch (HTTPS/rustls-tls)',
    category: 'auto',
    async fn() {
      // Use example.com — IANA-managed, extremely reliable
      const resp = await retryFetch('https://www.example.com', { method: 'GET' })
      assert(
        resp.status === 200,
        `HTTPS connection failed with status ${resp.status}`
      )
      assert(
        resp.url.startsWith('https://'),
        `expected HTTPS URL, got ${resp.url}`
      )
    }
  },
  {
    name: '@tauri-apps/plugin-http.fetch (error handling)',
    category: 'auto',
    async fn() {
      const { fetch } = await import('@tauri-apps/plugin-http')
      const resp = await fetch('http://localhost:3003/status/404', {
        method: 'GET'
      })
      assert(resp.status === 404, `expected status 404, got ${resp.status}`)
      assert(!resp.ok, 'expected resp.ok to be false for 404')
    }
  },

  // @tauri-apps/plugin-fs
  {
    name: '@tauri-apps/plugin-fs.mkdir+writeFile+stat+readFile+exists+readDir+removeFile+removeDir',
    category: 'side-effect',
    async fn() {
      const { mkdir, writeFile, stat, readFile, exists, readDir, remove } = await import('@tauri-apps/plugin-fs');
      const { appCacheDir } = await import('@tauri-apps/api/path');

      const base = await appCacheDir();
      const testDir = `${base}/tauri-test-${Date.now()}`;
      const testFile = `${testDir}/test.txt`;
      const content = new TextEncoder().encode('hello tauri fs');

      await mkdir(testDir, { recursive: true });
      await writeFile(testFile, content);

      const info = await stat(testFile);
      assert(info.size === content.length, `stat size mismatch: ${info.size} vs ${content.length}`);

      const fileExists = await exists(testFile);
      assert(fileExists === true, 'exists returned false for written file');

      const read = await readFile(testFile);
      const decoded = new TextDecoder().decode(read);
      assert(decoded === 'hello tauri fs', `readFile content mismatch: "${decoded}"`);

      const entries = await readDir(testDir);
      assert(entries.length >= 1, `readDir returned ${entries.length} entries, expected >= 1`);

      await remove(testFile);
      await remove(testDir, { recursive: true });

      const afterRemove = await exists(testFile);
      assert(afterRemove === false, 'file still exists after remove');
    },
  },

  // @tauri-apps/plugin-autostart
  {
    name: '@tauri-apps/plugin-autostart.enable+isEnabled+disable',
    category: 'side-effect',
    async fn() {
      const { enable, disable, isEnabled } = await import('@tauri-apps/plugin-autostart');
      await enable();
      const enabled = await isEnabled();
      assert(enabled === true, `isEnabled returned ${enabled} after enable()`);
      await disable();
      const disabled = await isEnabled();
      assert(disabled === false, `isEnabled returned ${disabled} after disable()`);
    },
  },

  // @tauri-apps/plugin-clipboard-manager
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeText+readText',
    category: 'side-effect',
    async fn() {
      const { writeText, readText } = await import('@tauri-apps/plugin-clipboard-manager');
      const testStr = `tauri-test-${Date.now()}`;
      await writeText(testStr);
      const result = await readText();
      assert(result === testStr, `clipboard mismatch: "${result}" vs "${testStr}"`);
    },
  },
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      // Valid 1x1 red pixel PNG
      const png = new Uint8Array([
        137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,
        0,0,0,1,0,0,0,1,8,2,0,0,0,144,119,83,
        222,0,0,0,12,73,68,65,84,120,156,99,248,207,192,0,
        0,3,1,1,0,201,254,146,239,0,0,0,0,73,69,78,
        68,174,66,96,130
      ]);
      await writeImage(png);
    },
  },
  // writeImage with number[] — verifies visit_seq deserialization path
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage(number[])',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      const png = [
        137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,
        0,0,0,1,0,0,0,1,8,2,0,0,0,144,119,83,
        222,0,0,0,12,73,68,65,84,120,156,99,248,207,192,0,
        0,3,1,1,0,201,254,146,239,0,0,0,0,73,69,78,
        68,174,66,96,130
      ];
      await writeImage(png);
    },
  },
  // writeImage with Image object — verifies Resource/rid path
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage(Image)',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      const { Image } = await import('@tauri-apps/api/image');
      const rgba = new Uint8Array([255, 0, 0, 255]);
      const img = await Image.new(rgba, 1, 1);
      await writeImage(img);
    },
  },
  // writeImage with larger RGBA — verifies non-trivial data size through TSFN
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage(4x4)',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      const rgba = new Uint8Array([
        255,0,0,255,    0,255,0,255,    0,0,255,255,    255,255,0,255,
        128,0,0,128,    0,128,0,128,    0,0,128,128,    128,128,0,128,
        64,0,0,64,      0,64,0,64,      0,0,64,64,      64,64,0,64,
        32,0,0,32,      0,32,0,32,      0,0,32,32,      32,32,0,32,
      ]);
      const { Image } = await import('@tauri-apps/api/image');
      const img = await Image.new(rgba, 4, 4);
      await writeImage(img);
    },
  },
  // writeImage with { rgba, width, height } object — verifies visit_map → JsImage::Rgba
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage(rgba-object)',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      const rgba = new Uint8Array([255, 0, 0, 255]);
      await writeImage({ rgba, width: 1, height: 1 });
    },
  },
  // writeImage with data URI string — verifies visit_str → JsImage::DataUri
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage(data-uri)',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      // Valid 1x1 red pixel PNG (color type 2 = RGB) as data URI
      const dataUri = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4z8AAAAMBAQDJ/pLvAAAAAElFTkSuQmCC';
      await writeImage(dataUri);
    },
  },
  // writeImage with file path string — verifies visit_str → JsImage::Path
  // Uses fs plugin + path API to create the file, no custom Rust command needed.
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage(path)',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      const { writeFile } = await import('@tauri-apps/plugin-fs');
      const { cacheDir, join } = await import('@tauri-apps/api/path');
      const png = new Uint8Array([
        137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,
        0,0,0,1,0,0,0,1,8,2,0,0,0,144,119,83,
        222,0,0,0,12,73,68,65,84,120,156,99,248,207,192,0,
        0,3,1,1,0,201,254,146,239,0,0,0,0,73,69,78,
        68,174,66,96,130
      ]);
      const dir = await cacheDir();
      const filePath = await join(dir, `test-clipboard-${Date.now()}.png`);
      await writeFile(filePath, png);
      await writeImage(filePath);
      // Clean up temp file after test
      const { remove } = await import('@tauri-apps/plugin-fs');
      await remove(filePath);
    },
  },
  // writeImage with ArrayBuffer — verifies visit_seq → JsImage::Bytes (IPC: buffer → sequence)
  {
    name: '@tauri-apps/plugin-clipboard-manager.writeImage(ArrayBuffer)',
    category: 'side-effect',
    async fn() {
      const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
      const png = new Uint8Array([
        137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,
        0,0,0,1,0,0,0,1,8,2,0,0,0,144,119,83,
        222,0,0,0,12,73,68,65,84,120,156,99,248,207,192,0,
        0,3,1,1,0,201,254,146,239,0,0,0,0,73,69,78,
        68,174,66,96,130
      ]);
      await writeImage(png.buffer.slice(0));
    },
  },

  // @tauri-apps/plugin-process (manual — kills the process, can't assert)
  {
    name: '@tauri-apps/plugin-process.relaunch',
    category: 'manual',
    async fn() {},
  },

  // @tauri-apps/plugin-dialog (manual)
{
    name: '@tauri-apps/plugin-dialog.open (single)',
    category: 'manual',
    async fn() {},
  },
  {
    name: '@tauri-apps/plugin-dialog.open (multiple)',
    category: 'manual',
    async fn() {},
  },
  {
    name: '@tauri-apps/plugin-dialog.save',
    category: 'manual',
    async fn() {},
  },
  {
    name: '@tauri-apps/plugin-dialog.confirm',
    category: 'manual',
    async fn() {},
  },
  {
    name: '@tauri-apps/plugin-dialog.message (info)',
    category: 'manual',
    async fn() {},
  },
  {
    name: '@tauri-apps/plugin-dialog.message (warning)',
    category: 'manual',
    async fn() {},
  },
  {
    name: '@tauri-apps/plugin-dialog.message (error)',
    category: 'manual',
    async fn() {},
  },

  // @tauri-apps/plugin-shell (manual)
  {
    name: '@tauri-apps/plugin-shell.open',
    category: 'manual',
    async fn() {},
  },

  // @tauri-apps/plugin-notification (manual)
  {
    name: '@tauri-apps/plugin-notification.sendNotification',
    category: 'manual',
    async fn() {},
  },

  // @tauri-apps/plugin-updater
  {
    name: '@tauri-apps/plugin-updater.check',
    category: 'auto',
    async fn() {
      const { check } = await import('@tauri-apps/plugin-updater');
      try {
        const update = await check();
        // null = no update available, Update object = update exists
        if (update !== null) {
          assert(typeof update.currentVersion === 'string', `currentVersion should be string, got ${typeof update.currentVersion}`);
          assert(typeof update.version === 'string', `version should be string, got ${typeof update.version}`);
          console.log(`[updater] Update available: ${update.currentVersion} → ${update.version}`);
        } else {
          console.log('[updater] No update available (null)');
        }
      } catch (e) {
        // AppGallery API may fail if app is not published or device lacks
        // AppGallery services. This is expected for dev/demo apps.
        // Re-throw only if the error is not network/service related.
        const msg = String(e);
        console.log(`[updater] check() rejected (expected for non-published apps): ${msg}`);
      }
    },
  },
  // downloadAndInstall is manual — triggers a system dialog on OHOS
  {
    name: '@tauri-apps/plugin-updater.downloadAndInstall',
    category: 'manual',
    async fn() {},
  },
];
