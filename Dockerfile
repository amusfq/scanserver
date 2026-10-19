FROM node:24-trixie-slim AS ui-build
WORKDIR /app
COPY app-ui/package*.json ./
RUN npm clean-install
COPY app-ui/ ./
RUN npm run build

FROM rust:1-trixie AS rust-build
WORKDIR /workspace
COPY rust-server/Cargo.toml rust-server/Cargo.lock ./rust-server/
COPY rust-server/src ./rust-server/src
COPY app-server/data/preview/default.jpg ./app-server/data/preview/default.jpg
WORKDIR /workspace/rust-server
RUN cargo build --locked --release

FROM debian:trixie-slim AS scanservjs-base
RUN apt-get update \
  && apt-get install -yq \
    adduser \
    cups-client \
    imagemagick \
    ipp-usb \
    poppler-utils \
    sane-airscan \
    sane-utils \
    tesseract-ocr \
    tesseract-ocr-ces \
    tesseract-ocr-deu \
    tesseract-ocr-eng \
    tesseract-ocr-spa \
    tesseract-ocr-fra \
    tesseract-ocr-ita \
    tesseract-ocr-nld \
    tesseract-ocr-pol \
    tesseract-ocr-por \
    tesseract-ocr-rus \
    tesseract-ocr-tur \
    tesseract-ocr-chi-sim \
  && if [ -f /etc/ImageMagick-6/policy.xml ]; then sed -i 's/policy domain="coder" rights="none" pattern="PDF"/policy domain="coder" rights="read | write" pattern="PDF"/' /etc/ImageMagick-6/policy.xml; fi \
  && if [ -f /etc/ImageMagick-7/policy.xml ]; then sed -i 's/policy domain="coder" rights="none" pattern="PDF"/policy domain="coder" rights="read | write" pattern="PDF"/' /etc/ImageMagick-7/policy.xml; fi \
  && rm -rf /var/lib/apt/lists/*

FROM scanservjs-base AS scanservjs-core
ENV \
  SANED_NET_HOSTS="" \
  AIRSCAN_DEVICES="" \
  PIXMA_HOSTS="" \
  SCANIMAGE_LIST_IGNORE="" \
  DEVICES="" \
  OCR_LANG="" \
  SCANSERV_STATIC_DIR=/usr/lib/scanservjs/client \
  SCANSERV_DATA_DIR=/data

RUN mkdir -p /usr/lib/scanservjs/client /data
COPY --from=rust-build /workspace/rust-server/target/release/scanserver /usr/lib/scanservjs/scanserver
COPY --from=ui-build /app/dist/ /usr/lib/scanservjs/client/
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh /usr/lib/scanservjs/scanserver
WORKDIR /usr/lib/scanservjs
ENTRYPOINT ["/entrypoint.sh"]
EXPOSE 8080

FROM scanservjs-core AS scanservjs-user2001
ARG UID=2001
ARG GID=2001
ARG UNAME=scanservjs
RUN groupadd -g $GID -o $UNAME \
  && useradd -o -u $UID -g $GID -m -s /bin/bash $UNAME \
  && chown -R $UID:$GID /data /etc/sane.d
USER $UNAME

FROM scanservjs-core AS scanservjs-hplip
RUN apt-get update \
  && apt-get install -yq libsane-hpaio \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && echo hpaio >> /etc/sane.d/dll.conf

FROM scanservjs-core AS scanservjs-brscan4
RUN apt-get update \
  && apt-get install -yq curl \
  && curl -fSsL "https://download.brother.com/welcome/dlf105200/brscan4-0.4.11-1.amd64.deb" -o /tmp/brscan4.deb \
  && apt-get remove curl -yq \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && dpkg -i /tmp/brscan4.deb \
  && rm /tmp/brscan4.deb \
  && echo brscan4 >> /etc/sane.d/dll.conf
