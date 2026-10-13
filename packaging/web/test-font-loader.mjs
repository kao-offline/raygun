import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';

// Run the production reader rather than a copy of its validation rules.
const html = readFileSync(new URL('../../apps/lightcraft-web/index.html', import.meta.url), 'utf8');
const source = html.match(/    async function readChineseFont\(response\) \{[\s\S]*?\n    \}/)?.[0];
assert.ok(source, 'font reader exists');
const context = vm.createContext({Uint8Array, Error});
vm.runInContext(source + '\nglobalThis.readFont = readChineseFont;', context);
const readFont = context.readFont;
assert.deepEqual(await readFont(new Response(null, {status: 404})), new Uint8Array());
assert.deepEqual(await readFont(new Response(new Uint8Array([1, 2, 3]))), new Uint8Array([1, 2, 3]));
let cancelled = false;
const oversized = new ReadableStream({cancel() { cancelled = true; }});
await assert.rejects(readFont(new Response(oversized, {headers: {'Content-Length': String(33 * 1024 * 1024)}})), /32 MB/);
assert.ok(cancelled, 'declared oversized body is cancelled without reading');
cancelled = false;
let reads = 0;
const undeclared = new ReadableStream({
  pull(controller) { reads++; controller.enqueue(new Uint8Array(17 * 1024 * 1024)); },
  cancel() { cancelled = true; }
}, {highWaterMark: 0});
await assert.rejects(readFont(new Response(undeclared)), /32 MB/);
assert.equal(reads, 2);
assert.ok(cancelled, 'decoded stream is capped even without a trustworthy length');
console.log('ok: font reader preserves bytes and caps declared and streamed input');
