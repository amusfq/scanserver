<template>
  <div class="space-y-6">
    <header>
      <h1 class="text-2xl font-semibold tracking-tight sm:text-[28px]">{{ $t('settings.title') }}</h1>
    </header>

    <section class="card overflow-hidden rounded-2xl border border-base-300 bg-base-100 shadow-sm">
      <div class="card-body divide-y divide-base-300 gap-0 p-0">
      <div class="border-b border-base-300 px-4 py-4 sm:px-5">
        <h2 class="text-sm font-semibold">{{ $t('settings.behaviour-ui') }}</h2>
      </div>
      <label class="grid gap-3 px-4 py-4 sm:grid-cols-[1fr_14rem] sm:items-center sm:px-5">
        <span class="field-label">{{ $t('settings.theme') }}<span class="mt-1 block font-normal text-base-content/60">{{ $t('settings.theme:description') }}</span></span>
        <select v-model="settings.theme" class="select select-bordered w-full" @change="reload">
          <option v-for="item in themes" :key="item.value" :value="item.value">{{ item.text }}</option>
        </select>
      </label>
      <label class="grid gap-3 px-4 py-4 sm:grid-cols-[1fr_14rem] sm:items-center sm:px-5">
        <span class="field-label">{{ $t('settings.locale') }}<span class="mt-1 block font-normal text-base-content/60">{{ $t('settings.locale:description') }}</span></span>
        <select v-model="settings.locale" class="select select-bordered w-full" @change="reload">
          <option v-for="item in locales" :key="item.value" :value="item.value">{{ item.text }}</option>
        </select>
      </label>
      <label class="flex cursor-pointer items-center justify-between gap-4 px-4 py-4 sm:px-5">
        <span class="field-label">{{ $t('settings.show-files-after-scan') }}<span class="mt-1 block font-normal text-base-content/60">{{ $t('settings.show-files-after-scan:description') }}</span></span>
        <input v-model="settings.showFilesAfterScan" type="checkbox" class="checkbox checkbox-primary shrink-0">
      </label>
      </div>
    </section>

    <section class="card overflow-hidden rounded-2xl border border-base-300 bg-base-100 shadow-sm">
      <div class="card-body divide-y divide-base-300 gap-0 p-0">
      <div class="border-b border-base-300 px-4 py-4 sm:px-5">
        <h2 class="text-sm font-semibold">{{ $t('settings.devices') }}</h2>
      </div>
      <div class="flex flex-wrap items-center justify-between gap-4 px-4 py-4 sm:px-5">
        <div class="field-label">{{ $t('settings.reset') }}<p class="mt-1 font-normal text-base-content/60">{{ $t('settings.reset:description') }}</p></div>
        <button class="btn btn-outline" @click="reset">{{ $t('settings.reset') }}</button>
      </div>
      <div class="flex flex-wrap items-center justify-between gap-4 px-4 py-4 sm:px-5">
        <div class="field-label">{{ $t('settings.clear-storage') }}<p class="mt-1 font-normal text-base-content/60">{{ $t('settings.clear-storage:description') }}</p></div>
        <button class="btn btn-outline" @click="clearStorage">{{ $t('settings.clear-storage') }}</button>
      </div>
      </div>
    </section>
  </div>
</template>

<script>
import Common from '../classes/common';
import Constants from '../classes/constants';
import Storage from '../classes/storage';

const storage = Storage.instance();

export default {
  name: 'Settings',
  emits: ['mask', 'notify'],

  data() {
    return { settings: storage.settings };
  },

  computed: {
    locales() {
      return Constants.Locales.map(value => ({ text: this.$t(`locales.${value}`), value }));
    },
    themes() {
      return Object.values(Constants.Themes).map(value => ({ text: this.$t(`settings.theme:${value}`), value }));
    }
  },

  watch: {
    settings: {
      handler(settings) {
        storage.settings = settings;
      },
      deep: true
    }
  },

  methods: {
    clearStorage() {
      storage.request = null;
    },
    reload() {
      storage.settings = this.settings;
      location.href = `${location.pathname}?anticache=${Date.now()}`;
    },
    reset() {
      this.$emit('mask', 1);
      Common.fetch('api/v1/context', { method: 'DELETE' }).then(() => {
        this.reload();
      }).catch(error => {
        this.$emit('notify', { type: 'e', message: error });
        this.$emit('mask', -1);
      });
    }
  }
};
</script>
