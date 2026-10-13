# Raygun 0.2.1 deployment verification

Deployed 2026-10-13 to https://raygun.kaooffline.top using the existing Cloudflare
Worker `raygun-web` and `hosting/wrangler.jsonc`. Exact source revision:
`808436d9d5fa842e4dcc9ab084f6476481a30b36`.

- Release build: https://github.com/kao-offline/raygun/actions/runs/38218128278
  (all targets successful).
- Web ZIP SHA-256: `a396e46d72fab162cfb6de7104c269b7033f7d318dd20e6562097c1d3a25a2cc`.
- Active Worker version: `f1536f40-bdbf-4662-9976-ce94a2d5ccee`, 100% traffic.
- Previous version retained: `2675af86-e2d8-4c89-8648-20b0bfec6f51`.

Wrangler dry-run and deploy succeeded. The live HTML, JS, WASM, render worker,
Chinese font and favicon were fetched and hashed against the release bundle.
All returned HTTP 200 and `Cache-Control: no-cache`; WASM returned
`application/wasm`. Cloudflare redirects `/index.html` to `/`.

Real browser verification: fresh library empty, actual PNG import, exposure edit,
JPEG download with changed pixels, photo and edit retained after reload, Chinese
menus/filename visible, benchmark isolated from OPFS, worker count bounded,
no browser exceptions or save errors. Local timings and limits are recorded in
[performance-web.md](performance-web.md). Recovery logs, raw hashes, screenshots
and browser reports remain in ignored `target/release-review/`.

To roll back this deployment without changing the hostname route:

```sh
wrangler rollback 2675af86-e2d8-4c89-8648-20b0bfec6f51 --config hosting/wrangler.jsonc
```

Updates take effect on reload; browser-stored photos/edits remain in the same
origin. Browser storage can still be cleared by the browser or user. Desktop
Windows updates require administrator approval; macOS/Linux apply remains
manual. The user's live desktop installation was not changed during testing.
