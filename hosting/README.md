# Hosting Raygun web (WASM) on your own server via Portainer

## Option A — build from the release (no uploads)

1. Portainer > **Stacks** > **Add stack**, name it `raygun-web`.
2. Build method: **Repository**. URL: `https://github.com/kao-offline/raygun`,
   Compose path: `hosting/docker-compose.yml`.
3. Environment variables: `RAYGUN_VERSION=0.1.0`, `RAYGUN_PORT=8080`.
4. **Deploy the stack**. Open `http://<server>:8080`.

To update: bump `RAYGUN_VERSION` to the new release and Re-pull + redeploy.

## Option B — serve the zip you already downloaded

1. Unzip `raygun-web-<ver>.zip` from the GitHub release into a folder named
   `web/` next to a static file server (any `nginx:alpine` + `hosting/nginx.conf`
   works; see `packaging/web/` for Apache/Netlify header equivalents).
2. Point your reverse proxy at it. Required headers: `Content-Type:
   application/wasm` for `.wasm`, `Cache-Control: no-cache` on
   `.wasm`/`.js`/`.html`.

## Notes

- The app is fully static (no backend, no account). Any static host works:
  the Portainer stack above is just nginx.
- WebGPU in the browser is still in progress upstream; the WASM build renders
  on the CPU path where WebGPU is unavailable.
- Never expose Portainer itself to the internet without its own auth + TLS;
  put the app behind your usual reverse proxy (Traefik/NPM) with HTTPS.
