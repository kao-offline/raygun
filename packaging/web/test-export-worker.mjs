// Real browser regression: downloading alone is insufficient; Session destruction
// used to trap in WASM before the worker reported batch completion.
// Run with an installed Playwright module (no product dependency):
// PLAYWRIGHT_MODULE=/path/to/playwright/index.mjs node packaging/web/test-export-worker.mjs URL
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {readFile} from 'node:fs/promises';

const modulePath = process.env.PLAYWRIGHT_MODULE;
const origin = process.argv[2];
if (!modulePath || !origin) throw Error('Provide PLAYWRIGHT_MODULE and the web URL');
const {chromium} = await import(pathToFileURL(modulePath).href);
const browser = await chromium.launch({headless: true, ...(process.env.CHROME_PATH ? {executablePath: process.env.CHROME_PATH} : {})});
async function until(predicate, timeout = 60000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (await predicate()) return;
    await new Promise(resolve => setTimeout(resolve, 10));
  }
  throw Error('Browser condition did not become true');
}
try {
  for (const store of ['memory', 'opfs', 'idb']) {
    const context = await browser.newContext({acceptDownloads: true});
    try {
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      const command = (id, params = {}) => page.evaluate(async ({id, params}) =>
        JSON.parse(await lightcraft.command(id, JSON.stringify(params))), {id, params});
      const pixel = async file => page.evaluate(async bytes => {
        const image = await createImageBitmap(new Blob([new Uint8Array(bytes)], {type: 'image/jpeg'}));
        const canvas = document.createElement('canvas');
        canvas.width = image.width; canvas.height = image.height;
        const ctx = canvas.getContext('2d');
        ctx.drawImage(image, 0, 0);
        image.close();
        return Array.from(ctx.getImageData(24, 16, 1, 1).data);
      }, Array.from(await readFile(await file.path())));
      await page.goto(`${origin}/?store=${store}&workers=1`);
      await page.waitForFunction(() => globalThis.lightcraft && !document.getElementById('lightcraft_loading'), {}, {timeout: 120000});
      await page.evaluate(async () => {
        const canvas = document.createElement('canvas');
        canvas.width = 48; canvas.height = 32;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = '#6496c8'; ctx.fillRect(0, 0, 48, 32);
        const blob = await new Promise(resolve => canvas.toBlob(resolve, 'image/png'));
        globalThis.originalBytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
        const transfer = new DataTransfer();
        transfer.items.add(new File([blob], 'worker-regression.png', {type: 'image/png'}));
        document.querySelector('canvas').dispatchEvent(new DragEvent('drop', {dataTransfer: transfer, bubbles: true, cancelable: true}));
      });
      await until(async () => (await command('library.info')).photos === 1);
      await until(async () => (await command('library.selectAll')).selected === 1);
      // Simulate new deployment glue after this editor was opened. Disposable
      // export workers must keep using the page's original module/glue pair.
      await page.route('**/lightcraft_web.js', route => route.fulfill({
        contentType: 'text/javascript', body: 'throw new Error("wrong deployment glue loaded");',
      }));
      // Cancel while the worker is starting, then retry through the same UI path.
      await page.evaluate(async () => {
        const command = async (id, params = {}) => JSON.parse(await lightcraft.command(id, JSON.stringify(params)));
        const started = await command('app.export', {path: 'cancelled.jpg', format: 'jpeg'});
        if (!started.background) throw Error('Export did not start in the background');
        const task = (await command('activity.list')).tasks.find(task => task.kind === 'export');
        if (!task) throw Error('Export activity is missing');
        await command('activity.cancel', {id: task.id});
      });
      await until(async () => {
        const stats = await command('web.stats');
        return !stats.export && stats.lastExportResult;
      });
      assert.equal((await command('web.stats')).lastExportResult.cancelled, true);
      await page.route('**/export-worker.js', route => route.abort());
      assert.equal((await command('app.export', {path: 'failed.jpg', format: 'jpeg'})).background, true);
      await until(async () => {
        const stats = await command('web.stats');
        return !stats.export && stats.lastExportResult;
      });
      assert.match((await command('web.stats')).lastExportResult.error, /worker/i);
      await page.unroute('**/export-worker.js');
      const samples = [];
      for (const mode of [0, 3, 4]) {
        await command('develop.set', {control: 'light.toneMapper', value: mode});
        const [file, started] = await Promise.all([
          page.waitForEvent('download'),
          command('app.export', {path: 'worker-regression.jpg', format: 'jpeg'}),
        ]);
        assert.equal(started.background, true);
        assert.equal(await file.failure(), null);
        await until(async () => {
          const stats = await command('web.stats');
          return !stats.export && stats.lastExportResult;
        });
        const result = (await command('web.stats')).lastExportResult;
        assert.equal(result.error, undefined, JSON.stringify(result));
        assert.equal(result.cancelled, false);
        assert.equal(result.files.length, 1);
        assert.equal(result.files[0].width, 48);
        assert.equal(result.files[0].height, 32);
        samples.push(await pixel(file));
      }
      assert.notDeepEqual(samples[0], samples[1], 'AgX must change the rendered pixels');
      assert.notDeepEqual(samples[1], samples[2], 'Simplified ACES must use its own transform');
      const [original, sidecar] = await Promise.all([
        page.waitForEvent('download', file => file.suggestedFilename() === 'worker-original.png'),
        page.waitForEvent('download', file => file.suggestedFilename() === 'worker-original.xmp'),
        command('app.export', {path: 'worker-original.png', format: 'original'}),
      ]);
      const expected = await page.evaluate(() => originalBytes);
      assert.deepEqual(Array.from(await readFile(await original.path())), expected);
      assert.match(await readFile(await sidecar.path(), 'utf8'), /xmpmeta/);
      await until(async () => {
        const stats = await command('web.stats');
        return !stats.export && stats.lastExportResult;
      });
      const originalResult = (await command('web.stats')).lastExportResult;
      assert.equal(originalResult.error, undefined, JSON.stringify(originalResult));
      assert.equal(originalResult.files.length, 1);
      assert.equal(originalResult.files[0].sidecars.length, 1);
      if (store !== 'memory') {
        await until(async () => {
          const stats = await command('web.stats');
          return !stats.dirty && !stats.flushing;
        });
        await page.unroute('**/lightcraft_web.js');
        await page.reload();
        await page.waitForFunction(() => globalThis.lightcraft && !document.getElementById('lightcraft_loading'), {}, {timeout: 120000});
        assert.equal((await command('library.info')).photos, 1);
        await until(async () => (await command('library.selectAll')).selected === 1);
        const [restored] = await Promise.all([
          page.waitForEvent('download'),
          command('app.export', {path: 'restored.jpg', format: 'jpeg'}),
        ]);
        assert.equal(await restored.failure(), null);
        await until(async () => {
          const stats = await command('web.stats');
          return !stats.export && stats.lastExportResult;
        });
        const result = (await command('web.stats')).lastExportResult;
        assert.equal(result.error, undefined, JSON.stringify(result));
        assert.equal(result.files[0].width, 48);
        assert.equal(result.files[0].height, 32);
        assert.deepEqual(await pixel(restored), samples[2], 'Tone mapper edits must survive reload');
      }
      assert.deepEqual(errors, []);
      console.log(`${store}: export cancellation, startup failure, completion and retry passed`);
    } finally { await context.close(); }
  }
} finally { await browser.close(); }
