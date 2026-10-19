<template>
  <div class="space-y-6">
    <header>
      <h1 class="text-2xl font-semibold tracking-tight sm:text-[28px]">{{ $t('navigation.about') }}</h1>
    </header>

    <section class="card max-w-3xl rounded-2xl border border-base-300 bg-base-100 shadow-sm">
      <div class="card-body p-5 sm:p-6">
      <div class="flex flex-wrap items-baseline justify-between gap-2">
        <h2 class="text-lg font-semibold tracking-tight">{{ $t('global.application-name') }}</h2>
        <span class="text-xs text-base-content/60">v{{ version }}</span>
      </div>
      <p class="mt-4 text-sm leading-6 text-base-content/80">{{ $t('about.main') }}</p>
      <div class="mt-6 flex flex-wrap gap-x-5 gap-y-2 text-sm">
        <a class="font-medium text-primary underline-offset-4 hover:underline" href="https://github.com/amusfq/scanserver" target="_blank" rel="noreferrer">{{ $t('about.issue') }}</a>
      </div>
      <p class="mt-4 text-xs text-base-content/60">Based on <a class="underline underline-offset-4 hover:text-primary" href="https://github.com/sbs20/scanservjs" target="_blank" rel="noreferrer">scanservjs by Sam Strachan</a>.</p>
      <button class="btn btn-outline mt-7 self-start" @click="showSystemInfo">{{ $t('about.system-info') }}</button>
      </div>
    </section>

    <div v-if="systemInfoDialog" class="modal modal-open z-50" role="dialog" aria-modal="true" :aria-label="$t('about.system-info')" @click.self="systemInfoDialog = false" @keydown.esc.prevent="systemInfoDialog = false">
      <section class="modal-box max-w-xl overflow-hidden p-0">
        <header class="flex items-center justify-between border-b border-base-300 px-4 py-3.5 sm:px-5">
          <h2 class="text-sm font-semibold">{{ $t('about.system-info') }}</h2>
          <button class="btn btn-ghost btn-square btn-sm" :aria-label="$t('batch-dialog.btn-cancel')" @click="systemInfoDialog = false">×</button>
        </header>
        <pre class="max-h-[65vh] overflow-auto p-4 text-xs leading-5 text-base-content/80 sm:p-5">{{ JSON.stringify(systemInfo, null, 2) }}</pre>
      </section>
    </div>
  </div>
</template>

<script>
import Common from '../classes/common';
import Constants from '../classes/constants';

export default {
  name: 'About',
  emits: ['mask', 'notify'],

  data() {
    return {
      version: Constants.Version,
      systemInfo: null,
      systemInfoDialog: false
    };
  },

  methods: {
    showSystemInfo() {
      this.$emit('mask', 1);
      Common.fetch('api/v1/system').then(data => {
        this.systemInfo = data;
        this.systemInfoDialog = true;
        this.$emit('mask', -1);
      }).catch(error => {
        this.$emit('notify', { type: 'e', message: error });
        this.$emit('mask', -1);
      });
    }
  }
};
</script>
