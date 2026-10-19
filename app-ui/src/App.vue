<template>
  <div :class="['min-h-screen bg-canvas text-ink', { dark: darkMode }]" :data-theme="darkMode ? 'night' : 'corporate'" :dir="isRtl() ? 'rtl' : 'ltr'">
    <navigation />
    <main class="min-h-screen pb-24 lg:pb-10 lg:pl-64">
      <div class="mx-auto w-full max-w-screen-2xl px-4 py-5 sm:px-6 lg:px-10 lg:py-8">
        <router-view v-slot="{ Component }" @mask="mask" @notify="notify">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>

    <transition name="fade">
      <div v-if="maskRef" class="fixed inset-0 z-40 grid place-items-center bg-black/25 backdrop-blur-[2px]" aria-label="Loading">
        <div class="h-8 w-8 animate-spin rounded-full border-[3px] border-base-300 border-t-primary" />
      </div>
    </transition>

    <transition name="fade">
      <div v-if="notification" :class="['alert fixed bottom-20 left-4 right-4 z-50 shadow-lg sm:left-auto sm:right-6 sm:w-auto sm:max-w-md', notification.type === 'e' ? 'alert-error' : 'alert-info']" :role="notification.type === 'e' ? 'alert' : 'status'">
        {{ notification.message }}
      </div>
    </transition>
  </div>
</template>

<script>

import Constants from './classes/constants';
import ManifestBuilder from './classes/manifest-builder';
import Storage from './classes/storage';
import Navigation from './components/Navigation.vue';

const storage = Storage.instance();

export default {
  name: 'App',
  components: {
    Navigation,
  },
  data() {
    let theme = storage.settings.theme;
    if (theme === Constants.Themes.System) {
      theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? Constants.Themes.Dark : Constants.Themes.Light;
    }
    return {
      maskRef: 0,
      darkMode: theme === Constants.Themes.Dark,
      notification: null,
      notificationTimer: null
    };
  },

  beforeMount() {
    const locale = new URLSearchParams(window.location.search).get('locale')
      || storage.settings.locale
      || navigator.languages[0]
      || 'en';
    const settings = storage.settings;
    settings.locale = locale;
    storage.settings = settings;
  },

  mounted() {
    this.$i18n.locale = storage.settings.locale;
    const manifest = ManifestBuilder.create().withDark(this.darkMode).withStorage(storage).build();
    const element = document.createElement('link');
    element.setAttribute('rel', 'manifest');
    element.setAttribute('href', `data:manifest+json,${encodeURIComponent(JSON.stringify(manifest))}`);
    document.querySelector('head').appendChild(element);

    // Default route if connected
    if (this.$route.matched.length === 0) {
      this.$router.replace('/scan');
    }
  },

  methods: {
    isRtl() {
      return Constants.RtlLocales.includes(storage.settings.locale);
    },

    mask(add) {
      this.maskRef += add;
    },

    notify(notification) {
      window.clearTimeout(this.notificationTimer);
      this.notification = notification;
      this.notificationTimer = window.setTimeout(() => {
        this.notification = null;
      }, notification.type === 'e' ? 10000 : 2800);
    }
  }
};
</script>

<style>
.fade-enter-active, .fade-leave-active {
  transition: opacity .16s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>
