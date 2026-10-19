<template>
  <div class="space-y-6">
    <header class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight sm:text-[28px]">{{ $t('navigation.files') }}</h1>
      </div>
      <div class="flex items-center gap-2">
        <button :disabled="selectedFiles.length === 0" class="btn btn-outline btn-sm" @click="multipleDelete">{{ $t('files.button:delete-selected') }}<span v-if="selectedFiles.length"> · {{ selectedFiles.length }}</span></button>
        <button class="btn btn-ghost btn-square btn-sm" :aria-label="$t('navigation.files')" @click="fileList">
          <svg viewBox="0 0 24 24" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M20 7v5h-5M4 17v-5h5m10-1a7 7 0 0 0-12.7-3M5 13a7 7 0 0 0 12.7 3" /></svg>
        </button>
      </div>
    </header>

    <section class="card overflow-hidden rounded-2xl border border-base-300 bg-base-100 shadow-sm">
      <div class="flex flex-wrap items-center justify-between gap-4 border-b border-base-300 px-4 py-3.5 sm:px-5">
        <label class="flex items-center gap-2.5 text-xs font-medium text-base-content/80">
          <input v-model="thumbnails.show" type="checkbox" class="checkbox checkbox-primary checkbox-sm">
          {{ $t('files.thumbnail-show') }}
        </label>
        <label v-if="thumbnails.show" class="flex items-center gap-3 text-xs text-base-content/60">
          {{ $t('files.thumbnail-size') }}
          <input v-model.number="thumbnails.size" type="range" min="32" max="128" step="16" class="range range-primary range-sm w-28">
          <span class="w-7 tabular-nums">{{ thumbnails.size }}</span>
        </label>
      </div>

      <div v-if="!files.length" class="grid min-h-64 place-items-center px-6 py-14 text-center">
        <div>
          <div class="mx-auto grid h-12 w-12 place-items-center rounded-2xl bg-primary/10 text-primary">
            <svg viewBox="0 0 24 24" class="h-6 w-6" fill="none" stroke="currentColor" stroke-width="1.7"><path stroke-linecap="round" stroke-linejoin="round" d="M7 4h8l4 4v12H7zM15 4v5h4M4 8v13h12" /></svg>
          </div>
          <p class="mt-4 text-sm font-medium">{{ $t('files.empty') }}</p>
          <p class="mt-1 text-xs text-base-content/60">{{ $t('navigation.scan') }}</p>
        </div>
      </div>

      <div v-else class="divide-y divide-base-300">
        <div v-for="file in files" :key="file.name" class="flex items-center gap-3 px-4 py-3.5 transition hover:bg-base-200/60 sm:gap-4 sm:px-5">
          <input v-model="selectedFiles" type="checkbox" :value="file" class="checkbox checkbox-primary checkbox-sm shrink-0" :aria-label="file.name">
          <div v-if="thumbnails.show" class="grid shrink-0 place-items-center overflow-hidden rounded-lg border border-base-300 bg-base-200" :style="{ width: `${thumbnails.size}px`, height: `${thumbnails.size}px` }">
            <img :src="`api/v1/files/${encodeURIComponent(file.name)}/thumbnail`" class="max-h-full max-w-full object-contain" alt="" loading="lazy">
          </div>
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium text-ink">{{ file.name }}</div>
            <div class="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-xs text-base-content/60">
              <span>{{ $d(new Date(file.lastModified), 'short') }}</span>
              <span>{{ file.sizeString }}</span>
            </div>
          </div>
          <div class="flex shrink-0 items-center gap-1">
            <button v-if="['.pdf', '.jpg', '.jpeg', '.png', '.tif', '.tiff'].includes(file.extension.toLowerCase())" class="btn btn-ghost btn-square btn-sm text-primary" :aria-label="$t('navigation.print')" @click="print(file)">
              <svg viewBox="0 0 24 24" class="h-[18px] w-[18px]" fill="none" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M6 8V3h12v5M6 17H4V9h16v8h-2M7 14h10v7H7zM17 11h.01" /></svg>
            </button>
            <button class="btn btn-ghost btn-square btn-sm" :aria-label="$t('files.actions')" @click="open(file)">
              <svg viewBox="0 0 24 24" class="h-[18px] w-[18px]" fill="none" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M12 3v12m0 0 4-4m-4 4-4-4M5 17v3h14v-3" /></svg>
            </button>
            <button class="btn btn-ghost btn-sm hidden sm:inline-flex" @click="fileRename(file)">{{ $t('files.filename') }}</button>
            <button class="btn btn-ghost btn-square btn-sm text-error" :aria-label="$t('files.button:delete-selected')" @click="fileRemove(file)">
              <svg viewBox="0 0 24 24" class="h-[18px] w-[18px]" fill="none" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M4 7h16M10 11v6m4-6v6M6 7l1 13h10l1-13M9 7V4h6v3" /></svg>
            </button>
          </div>
        </div>
      </div>
    </section>

    <div v-if="dialogEdit" class="modal modal-open z-50" role="dialog" aria-modal="true" @click.self="closeRename">
      <div class="modal-box max-w-md">
        <h2 class="text-base font-semibold">{{ $t('files.dialog:rename') }}</h2>
        <label class="field-label mt-5 block">{{ $t('files.filename') }}
          <input v-model="editedItem.newName" class="input input-bordered mt-1.5 w-full" type="text" @keydown.enter="renameFileConfirm">
        </label>
        <div class="modal-action">
          <button class="btn btn-ghost" @click="closeRename">{{ $t('files.dialog:rename-cancel') }}</button>
          <button class="btn btn-primary" @click="renameFileConfirm">{{ $t('files.dialog:rename-save') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import Common from '../classes/common';
import Storage from '../classes/storage';
const storage = Storage.instance();

export default {
  name: 'Files',

  emits: ['mask', 'notify'],

  data() {
    return {
      dialogEdit: false,
      files: [],
      editedItem: {
        name: '',
        newName: ''
      },
      defaultItem: {
        name: '',
        newName: ''
      },
      selectedFiles: [],
      thumbnails: {
        show: storage.settings.thumbnails.show,
        size: storage.settings.thumbnails.size
      }
    };
  },

  watch: {
    dialogEdit(val) {
      val || this.closeRename();
    },
    thumbnails: {
      handler(thumbnails) {
        const settings = storage.settings;
        settings.thumbnails = thumbnails;
        storage.settings = settings;
      },
      deep: true
    }
  },

  mounted() {
    this.fileList();
  },

  methods: {
    fileList() {
      this.$emit('mask', 1);
      Common.fetch('api/v1/files').then(files => {
        this.files = files;
        this.$emit('mask', -1);
      }).catch(error => {
        this.$emit('notify', {type: 'e', message: error});
        this.$emit('mask', -1);
      });
    },

    fileRemove(file) {
      this.$emit('mask', 1);
      Common.fetch(`api/v1/files/${file.name}`, {
        method: 'DELETE'
      }).then(data => {
        this.$emit('notify', {type: 'i', message: `${this.$t('files.message:deleted', [data.name])}`});
        this.fileList();
        this.$emit('mask', -1);
      }).catch(error => {
        this.$emit('notify', {type: 'e', message: error});
        this.$emit('mask', -1);
      });
    },

    fileRename(file) {
      this.editedIndex = this.files.indexOf(file);
      this.editedItem = Object.assign({}, file);
      this.editedItem.newName = this.editedItem.name;
      this.dialogEdit = true;
    },

    renameFileConfirm() {
      this.$emit('mask', 1);
      Common.fetch(`api/v1/files/${this.editedItem.name}`, {
        method: 'PUT',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({newName: this.editedItem.newName})
      }).then(() => {
        this.$emit('notify', {type: 'i', message: `${this.$t('files.message:renamed')}`});
        this.fileList();
        this.$emit('mask', -1);
      }).catch(error => {
        this.$emit('notify', {type: 'e', message: error});
        this.$emit('mask', -1);
      }).finally(() => {
        this.closeRename();
      });
    },

    closeRename() {
      this.dialogEdit = false;
      this.$nextTick(() => {
        this.editedItem = Object.assign({}, this.defaultItem);
        this.editedIndex = -1;
      });
    },

    async multipleDelete() {
      let refresh = false;
      while (this.selectedFiles.length > 0) {
        refresh = true;
        const name = this.selectedFiles[0].name;
        try {
          await Common.fetch(`api/v1/files/${name}`, {method: 'DELETE'});
          this.$emit('notify', {type: 'i', message: `${this.$t('files.message:deleted', [name])}`});
        } catch (error) {
          this.$emit('notify', {type: 'e', message: error});
        }
        this.selectedFiles.splice(0, 1);
      }

      if (refresh) {
        this.fileList();
      }
    },

    open(file) {
      window.location.href = `api/v1/files/${encodeURIComponent(file.name)}`;
    },

    print(file) {
      this.$router.push({ path: '/print', query: { file: file.name } });
    }
  }
};
</script>
