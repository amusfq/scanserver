# Scanserver

Scanserver is a web interface for SANE scanners, forked from [scanservjs](https://github.com/sbs20/scanservjs). The backend is Rust; the responsive interface uses Vue, Tailwind CSS, and DaisyUI.

It supports scanner discovery, previews and crop selection, single and multipage scans, image filters, OCR, and managing saved scans. Your scanner must be supported by SANE on the host.

## Build and run

Requirements: Rust 1.85 or newer, Node.js 24, and the scanner utilities used at runtime (`sane-utils`, `imagemagick`, `poppler-utils`, and `tesseract-ocr`).

```sh
npm run bootstrap
npm run build
SCANSERV_STATIC_DIR="$PWD/dist/client" SCANSERV_DATA_DIR="$PWD/data" ./dist/scanserver
```

Open `http://localhost:8080`. For development, run `npm run dev:server` and `npm run dev` in separate terminals. The Vite dev server proxies API requests to port 8080.

The Debian package can be built after `npm run build` with `./makedeb.sh`; the package is written under `debian/`.

## Docker

```sh
docker build -t scanserver .
docker run --rm --privileged -p 8080:8080 -v scanserver-data:/data scanserver
```

## Printing

Scanserver sends PDF and image jobs through CUPS. It uses the CUPS default printer unless `PRINTER_QUEUE` is set. In Docker, point `CUPS_SERVER` at a CUPS server reachable from the container and set `PRINTER_QUEUE` to a queue on that server; for example, add `-e CUPS_SERVER=192.168.1.20:631 -e PRINTER_QUEUE=Epson_L3210_IPP` to `docker run`. The print page reads paper, media, color, and quality choices from that queue.

## Configuration

The server listens on `0.0.0.0:8080` by default. Set `SCANSERV_HOST`, `SCANSERV_PORT`, `SCANSERV_STATIC_DIR`, and `SCANSERV_DATA_DIR` to override those values. Set `CUPS_SERVER` and `PRINTER_QUEUE` to select a remote CUPS server and printer. SANE options such as `DEVICES`, `SANED_NET_HOSTS`, `AIRSCAN_DEVICES`, `PIXMA_HOSTS`, and `OCR_LANG` are also read from the environment.

Scanserver has no built-in login or TLS. Keep it on a trusted network or place it behind an authenticated HTTPS proxy.

## Scope

This fork keeps the main scan and file workflows. Upstream JavaScript configuration hooks, custom file actions, and the built-in OpenAPI explorer are not ported yet. The project remains GPL-2.0; this fork builds on work by Sam Strachan and the scanservjs contributors.
