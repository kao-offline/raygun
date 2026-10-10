# Builds nginx serving the Raygun WASM bundle straight from the GitHub release.
# No files to upload: the release zip is fetched at image build time.
ARG RAYGUN_VERSION=0.1.0
FROM alpine:3.21 AS fetch
ARG RAYGUN_VERSION
RUN apk add --no-cache unzip wget && \
    wget -O /tmp/web.zip "https://github.com/kao-offline/raygun/releases/download/v${RAYGUN_VERSION}/raygun-web-${RAYGUN_VERSION}.zip" && \
    mkdir -p /web && unzip -o /tmp/web.zip -d /web

FROM nginx:1.27-alpine
COPY --from=fetch /web/ /usr/share/nginx/html/
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
