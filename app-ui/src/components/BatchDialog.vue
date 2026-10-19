<template>
  <div v-if="show" class="modal modal-open z-50" role="dialog" aria-modal="true" @click.self="cancel" @keydown.esc.prevent="cancel" @keydown.enter.prevent="next">
    <section class="modal-box max-w-xl overflow-hidden p-0">
      <div class="border-b border-base-300 px-5 py-4">
        <p class="text-[10px] font-semibold uppercase tracking-[0.16em] text-primary">{{ $t('scan.batch') }}</p>
        <h2 class="mt-1 text-base font-semibold">{{ message }}</h2>
      </div>
      <div v-if="image" class="grid max-h-[55vh] place-items-center bg-base-200 p-4">
        <img :src="'data:image/jpeg;base64,' + image" class="max-h-[50vh] max-w-full rounded-lg object-contain" alt="Scanned page preview">
      </div>
      <div class="modal-action mx-5 mb-5 mt-0">
        <button class="btn btn-ghost mr-auto" @click="cancel">{{ $t('batch-dialog.btn-cancel') }}</button>
        <button v-if="onRescan" class="btn btn-outline" @click="rescan">{{ $t('batch-dialog.btn-rescan') }}</button>
        <button v-if="onFinish" class="btn btn-outline" @click="finish">{{ $t('batch-dialog.btn-finish') }}</button>
        <button class="btn btn-primary" @click="next">{{ $t('batch-dialog.btn-next') }}</button>
      </div>
    </section>
  </div>
</template>

<script>
export default {
  name: 'BatchDialog',
  data() {
    return {
      message: null,
      image: null,
      show: false,
      onFinish: null,
      onNext: null,
      onRescan: null
    };
  },
  methods: {
    finish() {
      this.show = false;
      this.onFinish?.();
    },
    rescan() {
      this.show = false;
      this.onRescan?.();
    },
    next() {
      this.show = false;
      this.onNext?.();
    },
    cancel() {
      this.show = false;
    },
    open(options) {
      this.message = options.message;
      this.image = options.image;
      this.onFinish = options.onFinish;
      this.onRescan = options.onRescan;
      this.onNext = options.onNext;
      this.show = true;
    }
  }
};
</script>
