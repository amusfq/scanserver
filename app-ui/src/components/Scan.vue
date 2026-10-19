<template>
  <div class="space-y-6">
    <header class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight sm:text-[28px]">{{ $t('navigation.scan') }}</h1>
      </div>
      <div class="badge badge-info badge-soft gap-2 border-0 px-3 py-3 text-xs">
        <span :class="['h-2 w-2 rounded-full', context.devices.length ? 'bg-primary' : 'bg-warning']" />
        {{ context.devices.length ? deviceLabel(device) : $t('global.no-data-text') }}
        <button class="btn btn-ghost btn-xs btn-square ml-1" :aria-label="$t('scan.device')" @click="deviceRefresh">
          <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M20 7v5h-5M4 17v-5h5m10-1a7 7 0 0 0-12.7-3M5 13a7 7 0 0 0 12.7 3" /></svg>
        </button>
      </div>
    </header>

    <div class="grid items-start gap-5 xl:grid-cols-[minmax(0,1fr)_340px]">
      <section class="card overflow-hidden rounded-2xl border border-base-300 bg-base-100 shadow-sm">
        <div class="card-body gap-0 p-0">
        <div class="flex items-center justify-between border-b border-base-300 px-4 py-3.5 sm:px-5">
          <div>
            <h2 class="text-sm font-semibold">{{ $t('scan.btn-preview') }}</h2>
          </div>
          <button v-if="geometry" class="btn btn-outline btn-sm" @click="createPreview">
            <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="10.8" cy="10.8" r="6.8" /><path stroke-linecap="round" d="m16 16 4.5 4.5M10.8 7.5v6.6M7.5 10.8h6.6" /></svg>
            {{ $t('scan.btn-preview') }}
          </button>
        </div>

        <div class="grid min-h-[420px] place-items-center bg-base-200 p-4 sm:min-h-[620px] sm:p-8">
          <div class="w-full" :style="{ maxWidth: `${preview.width}px` }">
            <cropper v-if="geometry" ref="cropper" :key="preview.key" class="cropper overflow-hidden rounded-lg shadow-lg" :transition-time="10" :wheel-resize="false"
              :default-position="cropperDefaultPosition" :default-size="cropperDefaultSize"
              :src="img" @change="onCropperChange" />
            <img v-else :src="img" class="mx-auto max-h-[70vh] w-full rounded-lg object-contain shadow-lg" alt="Scanner preview">
          </div>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-3 border-t border-base-300 px-4 py-3 sm:px-5">
          <p class="text-xs text-base-content/60">{{ geometry ? $t('scan.width') + ' × ' + $t('scan.height') : $t('scan.source') }}</p>
          <button class="btn btn-ghost btn-sm" @click="deletePreview">
            <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M4 7h16M10 11v6m4-6v6M6 7l1 13h10l1-13M9 7V4h6v3" /></svg>
            {{ $t('scan.btn-clear') }}
          </button>
        </div>
        </div>
      </section>

      <aside class="space-y-4">
        <section class="card rounded-2xl border border-base-300 bg-base-100 shadow-sm">
          <div class="card-body p-4 sm:p-5">
          <div class="mb-4 flex items-center justify-between">
            <h2 class="text-sm font-semibold">{{ $t('scan.device') }}</h2>
            <span class="text-[11px] text-base-content/60">{{ context.devices.length }} {{ context.devices.length === 1 ? 'device' : 'devices' }}</span>
          </div>
          <select v-if="context.devices.length" v-model="device" class="select select-bordered w-full" @change="clear">
            <option v-for="item in context.devices" :key="item.id" :value="item">{{ deviceLabel(item) }}</option>
          </select>
          <div v-else class="alert alert-warning alert-soft py-2 text-sm">{{ $t('scan.message:no-devices') }}</div>

          <div class="mt-5 grid grid-cols-2 gap-3">
            <label v-if="'--source' in device.features" class="field-label">
              {{ $t('scan.source') }}
              <select v-model="request.params.source" class="select select-bordered mt-1.5 w-full">
                <option v-for="item in sources" :key="item.value" :value="item.value">{{ item.text }}</option>
              </select>
            </label>
            <label v-if="'--mode' in device.features" class="field-label">
              {{ $t('scan.mode') }}
              <select v-model="request.params.mode" class="select select-bordered mt-1.5 w-full">
                <option v-for="item in modes" :key="item.value" :value="item.value">{{ item.text }}</option>
              </select>
            </label>
            <label class="field-label">
              {{ $t('scan.resolution') }}
              <select v-model.number="request.params.resolution" class="select select-bordered mt-1.5 w-full">
                <option v-for="resolution in device.features['--resolution'].options" :key="resolution" :value="resolution">{{ resolution }} dpi</option>
              </select>
            </label>
            <label class="field-label">
              {{ $t('scan.format') }}
              <select v-model="request.pipeline" class="select select-bordered mt-1.5 w-full">
                <option v-for="item in pipelines" :key="item.value" :value="item.value">{{ item.text }}</option>
              </select>
            </label>
          </div>

          <label v-if="'--adf-mode' in device.features" class="field-label mt-4 block">
            {{ $t('scan.adf-mode') }}
            <select v-model="request.params.adfMode" class="select select-bordered mt-1.5 w-full">
              <option v-for="item in adfModes" :key="item.value" :value="item.value">{{ item.text }}</option>
            </select>
          </label>

          <label class="field-label mt-4 block">
            {{ $t('scan.batch') }}
            <select v-model="request.batch" class="select select-bordered mt-1.5 w-full">
              <option v-for="item in batchModes" :key="item.value" :value="item.value">{{ item.text }}</option>
            </select>
          </label>
          </div>
        </section>

        <section class="card rounded-2xl border border-base-300 bg-base-100 shadow-sm">
          <div class="card-body p-4 sm:p-5">
          <h2 class="text-sm font-semibold">{{ $t('scan.filters') }}</h2>
          <div class="mt-3 space-y-2.5">
            <label v-for="item in filters" :key="item.value" class="flex cursor-pointer items-center gap-2.5 text-[13px] text-base-content/80">
              <input v-model="request.filters" type="checkbox" :value="item.value" class="checkbox checkbox-primary checkbox-sm" @change="readPreview">
              {{ item.text }}
            </label>
          </div>
          </div>
        </section>

        <details class="collapse collapse-arrow rounded-2xl border border-base-300 bg-base-100 shadow-sm">
          <summary class="collapse-title text-sm font-semibold">{{ $t('scan.paperSize') }} &amp; {{ $t('scan.filters') }}</summary>
          <div class="collapse-content space-y-4 border-t border-base-300 px-4 py-4 sm:px-5">
            <label v-if="geometry" class="field-label block">
              {{ $t('scan.paperSize') }}
              <select v-model="paperSize" class="select select-bordered mt-1.5 w-full" @change="updatePaperSize(paperSize)">
                <option :value="null">{{ $t('global.no-data-text') }}</option>
                <option v-for="item in paperSizes" :key="item.name" :value="item">{{ item.name }}</option>
              </select>
            </label>
            <div v-if="geometry" class="grid grid-cols-2 gap-3">
              <label v-for="field in geometryFields" :key="field.key" class="field-label">
                {{ field.label }}
                <input v-model.number="request.params[field.key]" class="input input-bordered mt-1.5 w-full" type="number" step="any" @blur="onCoordinatesChange">
              </label>
            </div>
            <label v-for="field in lightFields" :key="field.key" class="field-label block">
              <span class="flex justify-between"><span>{{ field.label }}</span><span class="tabular-nums text-muted">{{ request.params[field.key] }}</span></span>
              <input v-model.number="request.params[field.key]" class="range range-primary range-sm mt-2 w-full" type="range" :min="field.feature.limits[0]" :max="field.feature.limits[1]" :step="field.feature.interval || 1">
            </label>
          </div>
        </details>

        <button class="btn btn-primary w-full rounded-xl" :disabled="!context.devices.length" @click="scan(1)">
          <svg viewBox="0 0 24 24" class="h-[18px] w-[18px]" fill="none" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M5 5h14v14H5zM8 9h8M8 12h8M8 15h5" /></svg>
          {{ $t('scan.btn-scan') }}
        </button>
      </aside>
    </div>

    <batch-dialog ref="batchDialog" />
  </div>
</template>

<script>
import { Cropper } from 'vue-advanced-cropper';
import { useI18n } from 'vue-i18n';
import BatchDialog from './BatchDialog.vue';

import Common from '../classes/common';
import Device from '../classes/device';
import Request from '../classes/request';
import Storage from '../classes/storage';

import 'vue-advanced-cropper/dist/style.css';

const storage = Storage.instance();

function round(n, dp) {
  const f = Math.pow(10, dp || 0);
  return Math.round(n * f) / f;
}

function sanitiseLocaleKey(s) {
  return s.toLowerCase().replace(/\[/g, '(').replace(/\]/g, ')');
}

export default {

  name: 'Scan',

  components: {
    Cropper,
    BatchDialog
  },

  emits: ['mask', 'notify'],

  setup() {
    const { te } = useI18n();
    return {
      te
    };
  },

  data() {
    const device = Device.default();
    device.name = this.$t('global.no-data-text');
    const request = Request.create(null, device);

    return {
      context: {
        devices: [
          device
        ],
        paperSizes: [],
        version: '0'
      },
      device: device,
      img: null,
      request: request,
      paperSize: null,
      preview: {
        timer: 0,
        width: 400,
        key: 0
      }
    };
  },

  computed: {
    geometry() {
      return ['-x', '-y', '-l', '-t'].every(s => s in this.device.features);
    },

    deviceSize() {
      return !this.geometry ? undefined : {
        width: this.device.features['-x'].limits[1],
        height: this.device.features['-y'].limits[1]
      };
    },

    geometryFields() {
      return [
        { key: 'top', label: this.$t('scan.top') },
        { key: 'left', label: this.$t('scan.left') },
        { key: 'width', label: this.$t('scan.width') },
        { key: 'height', label: this.$t('scan.height') }
      ];
    },

    lightFields() {
      return [
        { key: 'brightness', feature: this.device.features['--brightness'], label: this.$t('scan.brightness') },
        { key: 'contrast', feature: this.device.features['--contrast'], label: this.$t('scan.contrast') }
      ].filter(field => field.feature);
    },

    batchModes() {
      return this.device.settings.batchMode.options.map(mode => {
        const key = `batch-mode.${sanitiseLocaleKey(mode)}`;
        let translation = this.$t(key);
        return {
          text: translation === key ? mode : translation,
          value: mode
        };
      });
    },

    filters() {
      return this.device.settings.filters.options.map(f => {
        return {
          text: this.$t(f),
          value: f
        };
      });
    },

    modes() {
      return '--mode' in this.device.features
        ? this.device.features['--mode'].options.map(mode => {
          const key = `mode.${sanitiseLocaleKey(mode)}`;
          return {
            text: this.te(key) ? this.$t(key) : mode,
            value: mode
          };
        })
        : undefined;
    },

    adfModes() {
      return '--adf-mode' in this.device.features
        ? this.device.features['--adf-mode'].options.map(adfMode => {
          const key = `adf-mode.${sanitiseLocaleKey(adfMode)}`;
          return {
            text: this.te(key) ? this.$t(key) : adfMode,
            value: adfMode
          };
        })
        : undefined;
    },

    paperSizes() {
      if (!this.geometry) {
        return undefined;
      }

      const deviceSize = {
        x: this.device.features['-x'].limits[1],
        y: this.device.features['-y'].limits[1]
      };

      return this.context.paperSizes
        .filter(paper => paper.dimensions.x <= deviceSize.x && paper.dimensions.y <= deviceSize.y)
        .map(paper => {
          const variables = (paper.name.match(/@:[a-z-.]+/ig) || []).map(s => s.substr(2));
          variables.forEach(v => {
            paper.name = paper.name.replaceAll(`@:${v}`, this.$t(v));
          });
          return paper;
        });
    },

    pipelines() {
      return this.device.settings.pipeline.options.map(p => {
        const variables = (p.match(/@:[a-z-.]+/ig) || []).map(s => s.substr(2));
        let text = p;
        variables.forEach(v => {
          text = text.replaceAll(`@:${v}`, this.$t(v));
        });

        return {
          text: text,
          value: p
        };
      });
    },

    sources() {
      return '--source' in this.device.features
        ? this.device.features['--source'].options.map(source => {
          const key = `source.${sanitiseLocaleKey(source)}`;
          const x =  {
            text: this.te(key) ? this.$t(key) : source,
            value: source
          };
          return x;
        })
        : undefined;
    }
  },

  watch: {
    request: {
      handler(request) {
        storage.request = request;
      },
      deep: true
    }
  },

  mounted() {
    this._resizePreview();
    this.readContext().then(() => {
      this.readPreview();
    });
    window.addEventListener('resize', () => {
      clearTimeout(this.preview.timer);
      this.preview.timer = setTimeout(this._resizePreview, 100);
    });
  },

  methods: {
    deviceLabel(device) {
      const model = device.id.match(/:epsonscan2:([^:]+):/);
      return model ? `Epson ${model[1]}` : device.name;
    },

    _resizePreview() {
      const paperRatio = this.geometry
        ? this.deviceSize.width / this.deviceSize.height
        : 210 / 297;

      // This only makes a difference when the col-width="auto" - so md+
      const mdBreakpoint = 960;
      if (window.innerWidth >= mdBreakpoint) {
        const appbarHeight = 80;
        const availableWidth = window.innerWidth - 30;
        const availableHeight = window.innerHeight - appbarHeight - 15;
        const desiredWidth = availableHeight * paperRatio;
        this.preview.width = Math.min(availableWidth / 2, desiredWidth);
        this.preview.key += 1;
      }
    },

    _fetch(url, options) {
      this.mask(1);
      return Common.fetch(url, options)
        .then(data => {
          this.mask(-1);
          return data;
        })
        .catch(error => {
          this.notify({ type: 'e', message: error });
          this.mask(-1);
          return error;
        });
    },

    createPreview() {
      this.mask(1);

      // Keep reloading the preview image
      const timer = window.setInterval(this.readPreview, 1000);

      let data = Common.clone(this.request);

      this._fetch('api/v1/preview', {
        method: 'POST',
        body: JSON.stringify(data),
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        }
      }).then(() => {
        window.clearInterval(timer);

        // Some scanners don't create the preview until after the scan has finished.
        // Run preview one last time
        window.setTimeout(this.readPreview, 1000);
        this.mask(-1);
      }).catch(() => {
        this.mask(-1);
      });
    },

    deletePreview() {
      this.mask(1);
      Common.fetch('api/v1/preview', {
        method: 'DELETE'
      }).then(() => {
        this.notify({ type: 'i', message: this.$t('scan.message:deleted-preview') });
        this.readPreview();
        this.mask(-1);
      }).catch(error => {
        this.notify({ type: 'e', message: error });
        this.mask(-1);
      });
    },

    pixelsPerMm() {
      const scanner = this.deviceSize;

      // The preview image may not have perfectly scaled dimensions
      // because pixel counts are integers. So we report a horizontal
      // and vertical resolution
      const image = this.$refs.cropper.imageSize;
      return {
        x: image.width / scanner.width,
        y: image.height / scanner.height
      };
    },

    scaleCoordinates(coordinates, xScale, yScale) {
      return {
        width: round(coordinates.width * xScale, 1),
        height: round(coordinates.height * yScale, 1),
        left: round(coordinates.left * xScale, 1),
        top: round(coordinates.top * yScale, 1)
      };
    },

    cropperDefaultPosition() {
      const adjusted = this.scaleCoordinates(
        this.request.params,
        this.pixelsPerMm().x,
        this.pixelsPerMm().y);

      return {
        left: adjusted.left,
        top: adjusted.top
      };
    },

    cropperDefaultSize() {
      const adjusted = this.scaleCoordinates(
        this.request.params,
        this.pixelsPerMm().x,
        this.pixelsPerMm().y);

      return {
        width: adjusted.width,
        height: adjusted.height
      };
    },

    mask(add) {
      this.$emit('mask', add);
    },

    notify(notification) {
      this.$emit('notify', notification);
    },

    onCoordinatesChange() {
      const adjusted = this.scaleCoordinates(
        this.request.params,
        this.pixelsPerMm().x,
        this.pixelsPerMm().y);

      this.$refs.cropper.setCoordinates(adjusted);
    },

    onCropperChange({ coordinates }) {
      const adjusted = this.scaleCoordinates(
        coordinates,
        1 / this.pixelsPerMm().x,
        1 / this.pixelsPerMm().y);

      // The cropper changes even when coordinates are set manually. This will
      // result in manually set values being overwritten because of rounding.
      // If someone is taking the trouble to set values manually then they
      // should be preserved. We should only update the values if they breach
      // a threshold or the scanner dimensions
      const scanner = this.deviceSize;
      const params = this.request.params;
      const threshold = 0.4;
      const boundAndRound = (n, min, max) => round(Math.min(Math.max(min, n), max), 1);
      const bestValue = (current, crop, min, max) => Math.abs(current - crop) < threshold
        ? boundAndRound(current, min, max)
        : boundAndRound(crop, min, max);

      params.width = bestValue(params.width, adjusted.width, 0, scanner.width);
      params.height = bestValue(params.height, adjusted.height, 0, scanner.height);
      params.left = bestValue(params.left, adjusted.left, 0, scanner.width);
      params.top = bestValue(params.top, adjusted.top, 0, scanner.height);
    },

    readContext() {
      // Only show notification if things are slow (first time / force)
      const timer = window.setTimeout(() => {
        this.notify({ type: 'i', message: this.$t('scan.message:loading-devices') });
      }, 250);

      return this._fetch('api/v1/context').then(context => {
        window.clearTimeout(timer);

        if (context.devices && context.devices.length > 0) {
          this.context = context;
          this.device = context.devices[0];
          this.request = this.buildRequest();
          for (let test of context.diagnostics) {
            if (!test.success) {
              this.notify({ type: 'e', message: test.message });
            }
          }
        } else {
          this.notify({ type: 'e', message: this.$t('scan.message:no-devices') });
        }
      });
    },

    deviceRefresh() {
      this._fetch('api/v1/context', {
        method: 'DELETE'
      }).then(() => {
        this.readContext();
      });
    },

    readPreview() {
      // Gets the preview image as a base64 encoded jpg and updates the UI
      const uri = 'api/v1/preview?' + new URLSearchParams(
        this.request.filters.map(e => ['filter', e]));

      this._fetch(uri, {
        cache: 'no-store',
        method: 'GET'
      }).then(data => {
        this.img = 'data:image/jpeg;base64,' + data.content;
        this._resizePreview();
      });
    },

    buildRequest() {
      let request = storage.request;
      if (request && request.params) {
        this.device = this.context.devices.filter(d => d.id === request.params.deviceId)[0]
          || this.context.devices[0];
      }

      request = Request.create(request, this.device);
      return request;
    },

    clear() {
      storage.request = null;
      this.request = this.buildRequest();
    },

    scan(index) {
      if (index !== undefined) {
        this.request.index = index;
      }

      const data = Common.clone(this.request);
      this._fetch('api/v1/scan', {
        method: 'POST',
        body: JSON.stringify(data),
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        }
      }).then((response) => {
        if (response && 'index' in response) {
          const options = {
            message: this.$t('scan.message:turn-documents'),
            onFinish: () => {
            },
            onNext: () => {
              this.request.index = response.index + 1;
              this.scan();
            }
          };
          if (response.image) {
            options.message = `${this.$t('scan.message:preview-of-page')} ${response.index}`;
            options.image = response.image;
            options.onFinish = () => {
              this.request.index = -1;
              this.scan();
            };
            options.onRescan = () => {
              this.request.index = response.index;
              this.scan();
            };
          }
          this.$refs.batchDialog.open(options);
        } else {
          // Finish
          if (storage.settings.showFilesAfterScan) {
            this.$router.push('/files');
          } else {
            this.readPreview();
          }
        }
      });
    },

    updatePaperSize(value) {
      if (value?.dimensions) {
        this.request.params.width = value.dimensions.x;
        this.request.params.height = value.dimensions.y;
        this.onCoordinatesChange();
      }
    }
  }
};
</script>
