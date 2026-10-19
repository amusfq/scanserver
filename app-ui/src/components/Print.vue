<template>
  <div class="space-y-6">
    <header class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight sm:text-[28px]">{{ $t('navigation.print') }}</h1>
        <p class="mt-1 text-sm text-base-content/60">{{ $t('print.subtitle') }}</p>
      </div>
      <span v-if="capabilities" class="badge badge-ghost">{{ capabilities.queue }}</span>
    </header>

    <div v-if="capabilityError" class="alert alert-warning text-sm" role="status">{{ capabilityError }}</div>

    <div class="join">
      <button :class="['btn join-item', mode === 'document' ? 'btn-primary' : 'btn-outline']" @click="setMode('document')">{{ $t('print.document') }}</button>
      <button :class="['btn join-item', mode === 'photos' ? 'btn-primary' : 'btn-outline']" @click="setMode('photos')">{{ $t('print.photo-sheet') }}</button>
    </div>

    <div class="grid items-start gap-5 xl:grid-cols-[minmax(0,1fr)_minmax(340px,0.8fr)]">
      <div class="space-y-5">
        <section v-if="mode === 'document'" class="card rounded-2xl border border-base-300 bg-base-100 p-5 shadow-sm sm:p-6">
          <h2 class="text-base font-semibold">{{ $t('print.document-source') }}</h2>
          <div class="mt-4 grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto]">
            <label class="field-label">
              {{ $t('print.saved-file') }}
              <select v-model="savedFileName" class="select select-bordered mt-1.5 w-full" @change="loadSavedPreview">
                <option value="">{{ $t('print.choose-saved-file') }}</option>
                <option v-for="file in printableFiles" :key="file.name" :value="file.name">{{ file.name }}</option>
              </select>
            </label>
            <label class="btn btn-outline self-end">
              {{ $t('print.upload-file') }}
              <input class="sr-only" type="file" accept=".pdf,.jpg,.jpeg,.png,.tif,.tiff" @change="chooseDocument">
            </label>
          </div>
          <div v-if="documentName" class="mt-4 flex items-center gap-3 rounded-xl bg-base-200 px-3 py-2.5">
            <div class="min-w-0 flex-1 truncate text-sm font-medium">{{ documentName }}</div>
            <button class="btn btn-ghost btn-xs" @click="clearDocument">{{ $t('print.remove-file') }}</button>
          </div>
          <div class="mt-5 grid gap-4 sm:grid-cols-2">
            <label v-if="capabilities?.media?.choices?.length" class="field-label">
              {{ $t('print.paper-size') }}
              <select v-model="settings.media" class="select select-bordered mt-1.5 w-full">
                <option v-for="value in documentMediaChoices" :key="value" :value="value">{{ mediaLabel(value) }}</option>
              </select>
            </label>
            <label class="field-label">
              {{ $t('print.copies') }}
              <input v-model.number="settings.copies" class="input input-bordered mt-1.5 w-full" type="number" min="1" max="99">
            </label>
          </div>
          <label class="mt-4 flex items-start gap-3 text-sm">
            <input v-model="settings.fitToPage" type="checkbox" class="checkbox checkbox-primary checkbox-sm mt-0.5">
            <span><span class="font-medium">{{ $t('print.fit-to-page') }}</span><span class="mt-0.5 block text-xs text-base-content/60">{{ $t('print.fit-to-page-help') }}</span></span>
          </label>
        </section>

        <section v-else class="card rounded-2xl border border-base-300 bg-base-100 p-5 shadow-sm sm:p-6">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 class="text-base font-semibold">{{ $t('print.photo-layout') }}</h2>
              <p class="mt-1 text-xs text-base-content/60">{{ $t('print.photo-layout-help') }}</p>
            </div>
            <label class="btn btn-primary btn-sm">
              {{ $t('print.add-photos') }}
              <input class="sr-only" type="file" accept="image/jpeg,image/png,image/webp" multiple @change="addPhotoFiles">
            </label>
          </div>

          <div class="mt-5 grid gap-3 sm:grid-cols-2">
            <label class="field-label">
              {{ $t('print.paper-size') }}
              <select v-model="settings.media" class="select select-bordered mt-1.5 w-full" @change="renderPhotoPreview">
                <option v-for="value in photoMediaChoices" :key="value" :value="value">{{ mediaLabel(value) }}</option>
              </select>
            </label>
            <label class="field-label">
              {{ $t('print.orientation') }}
              <select v-model="orientation" class="select select-bordered mt-1.5 w-full" @change="renderPhotoPreview">
                <option value="portrait">{{ $t('print.portrait') }}</option>
                <option value="landscape">{{ $t('print.landscape') }}</option>
              </select>
            </label>
          </div>

          <div class="mt-4 grid gap-3 sm:grid-cols-4">
            <label class="field-label">{{ $t('print.photo-width') }}
              <input v-model.number="photoWidth" class="input input-bordered mt-1.5 w-full" type="number" min="1" max="20" step="0.1" @input="renderPhotoPreview">
            </label>
            <label class="field-label">{{ $t('print.photo-height') }}
              <input v-model.number="photoHeight" class="input input-bordered mt-1.5 w-full" type="number" min="1" max="25" step="0.1" @input="renderPhotoPreview">
            </label>
            <label class="field-label">{{ $t('print.margin') }}
              <input v-model.number="margin" class="input input-bordered mt-1.5 w-full" type="number" min="0" max="30" step="0.5" @input="renderPhotoPreview">
            </label>
            <label class="field-label">{{ $t('print.gap') }}
              <input v-model.number="gap" class="input input-bordered mt-1.5 w-full" type="number" min="0" max="20" step="0.5" @input="renderPhotoPreview">
            </label>
          </div>

          <div class="mt-4 flex flex-wrap items-center justify-between gap-2 rounded-xl bg-base-200 px-3 py-2.5 text-xs">
            <div class="font-medium">{{ photoCount }} {{ $t('print.photo-copies') }} · {{ capacity }} {{ $t('print.per-page') }}</div>
            <div class="flex items-center gap-2">
              <button class="btn btn-ghost btn-xs" :disabled="pageIndex <= 0" :aria-label="$t('print.previous-page')" @click="changePage(-1)">‹</button>
              <span class="tabular-nums">{{ pageCount ? pageIndex + 1 : 0 }} / {{ pageCount }}</span>
              <button class="btn btn-ghost btn-xs" :disabled="pageIndex + 1 >= pageCount" :aria-label="$t('print.next-page')" @click="changePage(1)">›</button>
            </div>
          </div>

          <label v-if="savedImageFiles.length" class="field-label mt-4 block">
            {{ $t('print.add-saved-photo') }}
            <div class="mt-1.5 flex gap-2">
              <select v-model="savedPhotoName" class="select select-bordered min-w-0 flex-1">
                <option value="">{{ $t('print.choose-saved-file') }}</option>
                <option v-for="file in savedImageFiles" :key="file.name" :value="file.name">{{ file.name }}</option>
              </select>
              <button class="btn btn-outline" :disabled="!savedPhotoName || busy" @click="addSavedPhoto">{{ $t('print.add') }}</button>
            </div>
          </label>

          <div v-if="photos.length" class="mt-4 divide-y divide-base-300 rounded-xl border border-base-300">
            <div v-for="(photo, index) in photos" :key="photo.id" class="flex items-center gap-3 p-3">
              <img :src="photo.url" class="h-12 w-12 rounded-lg border border-base-300 object-cover" alt="">
              <div class="min-w-0 flex-1 truncate text-sm font-medium">{{ photo.name }}</div>
              <label class="sr-only" :for="`copies-${photo.id}`">{{ $t('print.copies') }}</label>
              <input :id="`copies-${photo.id}`" v-model.number="photo.copies" class="input input-bordered input-sm w-20" type="number" min="1" max="99" :aria-label="$t('print.copies')" @input="renderPhotoPreview">
              <div class="flex gap-1">
                <button class="btn btn-ghost btn-square btn-xs" :disabled="index === 0" :aria-label="$t('print.move-up')" @click="movePhoto(index, -1)">↑</button>
                <button class="btn btn-ghost btn-square btn-xs" :disabled="index + 1 === photos.length" :aria-label="$t('print.move-down')" @click="movePhoto(index, 1)">↓</button>
                <button class="btn btn-ghost btn-square btn-xs text-error" :aria-label="$t('print.remove-photo')" @click="removePhoto(index)">×</button>
              </div>
            </div>
          </div>
          <p v-if="photoCount > 200 || pageCount > 10" class="mt-3 text-xs text-error">{{ $t('print.photo-limit') }}</p>
        </section>

        <section class="card rounded-2xl border border-base-300 bg-base-100 p-5 shadow-sm sm:p-6">
          <div class="flex items-center justify-between gap-3">
            <h2 class="text-base font-semibold">{{ $t('print.quality') }} &amp; {{ $t('print.advanced') }}</h2>
          </div>
          <div class="mt-4 grid gap-4 sm:grid-cols-2">
            <label v-for="setting in advancedOptions" :key="setting.key" class="field-label">
              {{ $t(`print.${setting.label}`) }}
              <select v-model="settings[setting.key]" class="select select-bordered mt-1.5 w-full">
                <option v-for="value in setting.option.choices" :key="value" :value="value">{{ choiceLabel(value) }}</option>
              </select>
            </label>
          </div>
          <p v-if="!advancedOptions.length" class="mt-3 text-xs text-base-content/60">{{ $t('print.no-advanced-options') }}</p>
        </section>

        <button class="btn btn-primary btn-lg w-full rounded-xl" :disabled="!canPrint || busy" @click="print">
          <span v-if="busy" class="loading loading-spinner loading-sm"></span>
          {{ busy ? $t('print.submitting') : $t('print.send-to-printer') }}
        </button>
      </div>

      <section class="card overflow-hidden rounded-2xl border border-base-300 bg-base-100 shadow-sm xl:sticky xl:top-20">
        <div class="flex items-center justify-between border-b border-base-300 px-4 py-3.5">
          <div>
            <h2 class="text-sm font-semibold">{{ $t('print.preview') }}</h2>
            <p class="mt-0.5 text-xs text-base-content/60">{{ mode === 'photos' ? $t('print.preview-photo-note') : documentName || $t('print.preview-document-note') }}</p>
          </div>
          <span v-if="mode === 'photos' && pageCount" class="badge badge-ghost">{{ pageIndex + 1 }} / {{ pageCount }}</span>
        </div>
        <div class="grid min-h-[360px] place-items-center bg-base-200 p-5 sm:p-8">
          <canvas v-if="mode === 'photos' && capacity" ref="photoPreview" class="max-h-[68vh] w-full max-w-[520px] border border-base-300 bg-white shadow-lg" :width="previewWidth" :height="previewHeight" :aria-label="$t('print.preview')"></canvas>
          <div v-else-if="mode === 'document' && documentPreviewUrl" class="w-full overflow-hidden rounded-lg bg-white shadow-lg">
            <iframe v-if="documentIsPdf" :src="documentPreviewUrl" class="aspect-[0.707] max-h-[68vh] w-full" :title="$t('print.preview')"></iframe>
            <img v-else :src="documentPreviewUrl" class="mx-auto max-h-[68vh] w-auto max-w-full object-contain" :alt="documentName">
          </div>
          <div v-else class="max-w-56 text-center text-sm text-base-content/50">
            <svg viewBox="0 0 24 24" class="mx-auto h-9 w-9" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="M7 3.75h7l4.25 4.5v12H7zM14 3.75v5h4.25M9.5 13h6m-6 3h6" /></svg>
            <p class="mt-3">{{ mode === 'photos' ? $t('print.add-photos-prompt') : $t('print.choose-document-prompt') }}</p>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script>
import { markRaw } from 'vue';
import Common from '../classes/common';
import photoSheetPdf from '../classes/photo-sheet-pdf';

const imageExtensions = ['jpg', 'jpeg', 'png', 'webp'];
const documentExtensions = ['pdf', 'jpg', 'jpeg', 'png', 'tif', 'tiff'];
const maxPhotoBytes = 80 * 1024 * 1024;
const mediaDimensions = {
  a2: [420, 594], a3: [297, 420], a4: [210, 297], a5: [148, 210], a6: [105, 148],
  'a3+': [329, 483], '8k': [270, 390], '16k': [195, 270],
  b3: [353, 500], b4: [250, 353], b5: [176, 250], b6: [125, 176],
  letter: [215.9, 279.4], legal: [215.9, 355.6], tabloid: [279.4, 431.8],
  '3.5x5': [88.9, 127], '4x6': [101.6, 152.4], '5x7': [127, 177.8], '5x8': [127, 203.2], '8x10': [203.2, 254],
  fanfoldgermanlegal: [215.9, 330.2], '169widesize': [101.6, 180.6],
  indianlegal: [215.9, 355.6], postcard: [100, 148], 'env10': [104.8, 241.3], 'envdl': [110, 220], 'envc6': [114, 162],
  envelope_10: [104.8, 241.3], envelopedl: [110, 220], envelopec6: [114, 162]
};

function dimensionsFor(value) {
  const label = value.replace(/^Custom\./, '').toLowerCase();
  if (mediaDimensions[label]) return mediaDimensions[label];
  const match = label.match(/^(\d+(?:\.\d+)?)x(\d+(?:\.\d+)?)mm$/);
  if (match) return [Number(match[1]), Number(match[2])];
  const inches = label.match(/^(\d+(?:\.\d+)?)x(\d+(?:\.\d+)?)in$/);
  return inches ? [Number(inches[1]) * 25.4, Number(inches[2]) * 25.4] : null;
}

export default {
  name: 'Print',
  emits: ['mask', 'notify'],
  data() {
    return {
      mode: 'document', capabilities: null, capabilityError: '', files: [], savedFileName: '',
      localFile: null, documentPreviewUrl: '', documentPreviewIsOwned: false, documentPreviewRequest: 0,
      settings: { media: '', mediaType: '', color: '', quality: '', duplex: '', borderless: '', copies: 1, fitToPage: false },
      photos: [], savedPhotoName: '', orientation: 'portrait', photoWidth: 3, photoHeight: 4,
      margin: 5, gap: 3, pageIndex: 0, busy: false, nextPhotoId: 1
    };
  },
  computed: {
    printableFiles() {
      return this.files.filter(file => documentExtensions.includes(file.extension.replace('.', '').toLowerCase()));
    },
    savedImageFiles() {
      return this.files.filter(file => imageExtensions.includes(file.extension.replace('.', '').toLowerCase()));
    },
    savedFile() {
      return this.printableFiles.find(file => file.name === this.savedFileName);
    },
    documentMediaChoices() {
      return (this.capabilities?.media?.choices || []).filter(value => !value.startsWith('Custom.'));
    },
    documentName() {
      return this.localFile?.name || this.savedFile?.name || '';
    },
    documentIsPdf() {
      return this.documentName.toLowerCase().endsWith('.pdf');
    },
    photoMediaChoices() {
      return (this.capabilities?.media?.choices || []).filter(value => dimensionsFor(value));
    },
    paperDimensions() {
      const dimensions = dimensionsFor(this.settings.media) || [210, 297];
      return this.orientation === 'landscape' ? [dimensions[1], dimensions[0]] : dimensions;
    },
    capacity() {
      if (this.capabilities?.media?.choices?.length && !this.photoMediaChoices.length) return 0;
      const [width, height] = this.paperDimensions;
      const photoWidth = Number(this.photoWidth);
      const photoHeight = Number(this.photoHeight);
      if (![width, height, photoWidth, photoHeight, this.margin, this.gap].every(Number.isFinite) || photoWidth <= 0 || photoHeight <= 0 || this.margin < 0 || this.gap < 0) return 0;
      const columns = Math.floor((width - 2 * this.margin + this.gap) / (photoWidth + this.gap));
      const rows = Math.floor((height - 2 * this.margin + this.gap) / (photoHeight + this.gap));
      return Math.max(0, columns) * Math.max(0, rows);
    },
    photoCount() {
      return this.photos.reduce((sum, photo) => sum + Math.max(0, Number(photo.copies) || 0), 0);
    },
    pageCount() {
      return this.capacity ? Math.ceil(this.photoCount / this.capacity) : 0;
    },
    previewWidth() {
      return this.paperDimensions[0] > 0 ? Math.round(720 * this.paperDimensions[0] / Math.max(...this.paperDimensions)) : 720;
    },
    previewHeight() {
      return this.paperDimensions[1] > 0 ? Math.round(720 * this.paperDimensions[1] / Math.max(...this.paperDimensions)) : 720;
    },
    advancedOptions() {
      return [
        ['mediaType', 'media-type', this.capabilities?.mediaType],
        ['color', 'color-mode', this.capabilities?.color],
        ['quality', 'quality', this.capabilities?.quality],
        ['duplex', 'two-sided', this.capabilities?.duplex],
        ['borderless', 'borderless', this.capabilities?.borderless]
      ].filter(([, , option]) => option?.choices?.length).map(([key, label, option]) => ({ key, label, option }));
    },
    canPrint() {
      if (!this.capabilities || this.busy) return false;
      return this.mode === 'document'
        ? Boolean(this.localFile || this.savedFile)
        : this.photoCount > 0 && this.photoCount <= 200 && this.capacity > 0 && this.pageCount <= 10;
    }
  },
  watch: {
    photos: { deep: true, handler() { this.pageIndex = Math.min(this.pageIndex, Math.max(0, this.pageCount - 1)); this.renderPhotoPreview(); } }
  },
  async mounted() {
    await Promise.all([this.loadCapabilities(), this.loadFiles()]);
    const requested = this.$route.query.file;
    if (typeof requested === 'string' && this.printableFiles.some(file => file.name === requested)) {
      this.savedFileName = requested;
      await this.loadSavedPreview();
    }
  },
  beforeUnmount() {
    this.releaseDocumentPreview();
    for (const photo of this.photos) URL.revokeObjectURL(photo.url);
  },
  methods: {
    async loadCapabilities() {
      try {
        this.capabilities = await Common.fetch('api/v1/print/capabilities');
        for (const [key, option] of Object.entries({
          media: this.capabilities.media, mediaType: this.capabilities.mediaType, color: this.capabilities.color,
          quality: this.capabilities.quality, duplex: this.capabilities.duplex, borderless: this.capabilities.borderless
        })) this.settings[key] = option?.default || option?.choices?.[0] || '';
        if (this.capabilities.media && !this.documentMediaChoices.includes(this.settings.media)) {
          this.settings.media = this.documentMediaChoices[0] || '';
        }
        if (this.photoMediaChoices.length && !this.photoMediaChoices.includes(this.settings.media)) {
          this.settings.media = this.photoMediaChoices[0];
        }
        this.renderPhotoPreview();
      } catch (error) {
        this.capabilityError = this.errorMessage(error);
      }
    },
    async loadFiles() {
      try {
        this.files = await Common.fetch('api/v1/files');
      } catch (error) {
        this.$emit('notify', { type: 'e', message: this.errorMessage(error) });
      }
    },
    mediaLabel(value) {
      return value.replaceAll('_', ' ');
    },
    choiceLabel(value) {
      return value.replaceAll('_', ' ');
    },
    setMode(mode) {
      this.mode = mode;
      if (mode === 'photos') this.renderPhotoPreview();
    },
    chooseDocument(event) {
      const file = event.target.files?.[0];
      event.target.value = '';
      if (!file) return;
      const extension = file.name.split('.').pop().toLowerCase();
      if (!documentExtensions.includes(extension) || file.size > 80 * 1024 * 1024) {
        this.$emit('notify', { type: 'e', message: this.$t('print.invalid-file') });
        return;
      }
      this.savedFileName = '';
      this.localFile = file;
      this.setDocumentPreview(URL.createObjectURL(file));
    },
    async loadSavedPreview() {
      this.localFile = null;
      this.releaseDocumentPreview();
      const file = this.savedFile;
      if (!file) return;
      const requestId = ++this.documentPreviewRequest;
      try {
        const response = await fetch(`api/v1/files/${encodeURIComponent(file.name)}`);
        if (!response.ok) throw new Error(`Could not load ${file.name}`);
        const blob = await response.blob();
        if (requestId === this.documentPreviewRequest) this.setDocumentPreview(URL.createObjectURL(blob));
      } catch (error) {
        this.$emit('notify', { type: 'e', message: this.errorMessage(error) });
      }
    },
    setDocumentPreview(url) {
      this.releaseDocumentPreview();
      this.documentPreviewUrl = url;
      this.documentPreviewIsOwned = true;
    },
    releaseDocumentPreview() {
      if (this.documentPreviewIsOwned && this.documentPreviewUrl) URL.revokeObjectURL(this.documentPreviewUrl);
      this.documentPreviewUrl = '';
      this.documentPreviewIsOwned = false;
      this.documentPreviewRequest += 1;
    },
    clearDocument() {
      this.savedFileName = '';
      this.localFile = null;
      this.releaseDocumentPreview();
    },
    async addPhotoFiles(event) {
      const files = Array.from(event.target.files || []);
      event.target.value = '';
      for (const file of files) {
        const extension = file.name.split('.').pop().toLowerCase();
        if (this.photos.length >= 50) {
          this.$emit('notify', { type: 'e', message: this.$t('print.photo-count-limit') });
          break;
        }
        const totalBytes = this.photos.reduce((sum, photo) => sum + photo.fileSize, file.size);
        if (!imageExtensions.includes(extension) || totalBytes > maxPhotoBytes) {
          this.$emit('notify', { type: 'e', message: this.$t('print.invalid-photo') });
          continue;
        }
        const url = URL.createObjectURL(file);
        try {
          const image = markRaw(new Image());
          image.src = url;
          await image.decode();
          if (image.width * image.height > 60000000) throw new Error('Photo is too large to edit');
          this.photos.push({ id: this.nextPhotoId++, name: file.name, url, image, fileSize: file.size, copies: 1 });
        } catch {
          URL.revokeObjectURL(url);
          this.$emit('notify', { type: 'e', message: this.$t('print.invalid-photo') });
        }
      }
    },
    async addSavedPhoto() {
      const file = this.savedImageFiles.find(item => item.name === this.savedPhotoName);
      if (!file) return;
      try {
        const response = await fetch(`api/v1/files/${encodeURIComponent(file.name)}`);
        if (!response.ok) throw new Error(`Could not load ${file.name}`);
        const blob = await response.blob();
        const imageFile = new File([blob], file.name, { type: blob.type });
        await this.addPhotoFiles({ target: { files: [imageFile], value: '' } });
        this.savedPhotoName = '';
      } catch (error) {
        this.$emit('notify', { type: 'e', message: this.errorMessage(error) });
      }
    },
    removePhoto(index) {
      URL.revokeObjectURL(this.photos[index].url);
      this.photos.splice(index, 1);
    },
    movePhoto(index, direction) {
      const other = index + direction;
      if (other < 0 || other >= this.photos.length) return;
      this.photos.splice(other, 0, ...this.photos.splice(index, 1));
    },
    changePage(amount) {
      this.pageIndex = Math.max(0, Math.min(this.pageCount - 1, this.pageIndex + amount));
      this.renderPhotoPreview();
    },
    photoPlacements(pageIndex) {
      const [pageWidth, pageHeight] = this.paperDimensions;
      const columns = Math.floor((pageWidth - 2 * this.margin + this.gap) / (this.photoWidth + this.gap));
      const rows = Math.floor((pageHeight - 2 * this.margin + this.gap) / (this.photoHeight + this.gap));
      const count = Math.max(0, columns * rows);
      const blockWidth = columns * this.photoWidth + Math.max(0, columns - 1) * this.gap;
      const blockHeight = rows * this.photoHeight + Math.max(0, rows - 1) * this.gap;
      const xStart = (pageWidth - blockWidth) / 2;
      const yStart = (pageHeight - blockHeight) / 2;
      const start = pageIndex * count;
      const end = start + count;
      const placements = [];
      let cursor = 0;
      for (const photo of this.photos) {
        const copies = Math.max(0, Math.min(99, Number(photo.copies) || 0));
        const first = Math.max(0, start - cursor);
        const last = Math.min(copies, end - cursor);
        for (let copy = first; copy < last; copy += 1) {
          const index = cursor + copy - start;
          placements.push({
            photo,
            x: xStart + (index % columns) * (this.photoWidth + this.gap),
            y: yStart + Math.floor(index / columns) * (this.photoHeight + this.gap)
          });
        }
        cursor += copies;
        if (cursor >= end) break;
      }
      return placements;
    },
    drawPhotoPage(canvas, pageIndex) {
      const [widthMm, heightMm] = this.paperDimensions;
      const context = canvas.getContext('2d');
      const scale = canvas.width / widthMm;
      context.fillStyle = '#fff';
      context.fillRect(0, 0, canvas.width, canvas.height);
      context.imageSmoothingEnabled = true;
      context.imageSmoothingQuality = 'high';
      for (const { photo, x, y } of this.photoPlacements(pageIndex)) {
        const width = this.photoWidth * scale;
        const height = this.photoHeight * scale;
        const sourceRatio = photo.image.width / photo.image.height;
        const targetRatio = width / height;
        const sourceWidth = sourceRatio > targetRatio ? photo.image.height * targetRatio : photo.image.width;
        const sourceHeight = sourceRatio > targetRatio ? photo.image.height : photo.image.width / targetRatio;
        const sourceX = (photo.image.width - sourceWidth) / 2;
        const sourceY = (photo.image.height - sourceHeight) / 2;
        context.drawImage(photo.image, sourceX, sourceY, sourceWidth, sourceHeight, x * scale, y * scale, width, height);
      }
    },
    async renderPhotoPreview() {
      await this.$nextTick();
      const canvas = this.$refs.photoPreview;
      if (canvas && this.mode === 'photos' && this.capacity) this.drawPhotoPage(canvas, this.pageIndex);
    },
    async print() {
      if (!this.canPrint) return;
      this.busy = true;
      this.$emit('mask', 1);
      try {
        let result;
        if (this.mode === 'document' && this.savedFile && !this.localFile) {
          result = await Common.fetch(`api/v1/files/${encodeURIComponent(this.savedFile.name)}/print`, {
            method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(this.printSettings())
          });
        } else {
          let file = this.localFile;
          if (this.mode === 'photos') file = await this.createPhotoPdf();
          const form = new FormData();
          form.append('file', file);
          form.append('settings', JSON.stringify(this.printSettings()));
          result = await Common.fetch('api/v1/print/upload', { method: 'POST', body: form });
        }
        this.$emit('notify', { type: 'i', message: result.message || this.$t('print.job-submitted') });
      } catch (error) {
        this.$emit('notify', { type: 'e', message: this.errorMessage(error) });
      } finally {
        this.busy = false;
        this.$emit('mask', -1);
      }
    },
    printSettings() {
      return {
        media: this.settings.media || undefined,
        mediaType: this.settings.mediaType || undefined,
        color: this.settings.color || undefined,
        quality: this.settings.quality || undefined,
        duplex: this.settings.duplex || undefined,
        borderless: this.settings.borderless || undefined,
        copies: Math.max(1, Math.min(99, Number(this.settings.copies) || 1)),
        fitToPage: this.mode === 'document' && this.settings.fitToPage
      };
    },
    async createPhotoPdf() {
      const [widthMm, heightMm] = this.paperDimensions;
      const dpi = 300;
      const images = [];
      let encodedBytes = 0;
      for (let index = 0; index < this.pageCount; index += 1) {
        const canvas = document.createElement('canvas');
        canvas.width = Math.round(widthMm * dpi / 25.4);
        canvas.height = Math.round(heightMm * dpi / 25.4);
        this.drawPhotoPage(canvas, index);
        const blob = await new Promise(resolve => canvas.toBlob(resolve, 'image/jpeg', 0.94));
        if (!blob) throw new Error(this.$t('print.pdf-error'));
        encodedBytes += blob.size;
        if (encodedBytes > 76 * 1024 * 1024) throw new Error(this.$t('print.pdf-too-large'));
        images.push({ width: canvas.width, height: canvas.height, jpeg: new Uint8Array(await blob.arrayBuffer()) });
        canvas.width = 0;
        canvas.height = 0;
      }
      const pdf = photoSheetPdf(images, widthMm, heightMm);
      if (pdf.size > 78 * 1024 * 1024) throw new Error(this.$t('print.pdf-too-large'));
      return new File([pdf], 'photo-sheet.pdf', { type: 'application/pdf' });
    },
    errorMessage(error) {
      try { return JSON.parse(error).message || String(error); } catch { return String(error); }
    }
  }
};
</script>
