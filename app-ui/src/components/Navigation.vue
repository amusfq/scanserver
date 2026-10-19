<template>
  <aside class="fixed inset-y-0 left-0 z-20 hidden w-64 flex-col border-r border-base-300 bg-base-100 px-5 py-6 lg:flex">
    <div class="flex items-center gap-3 px-2">
      <div class="grid h-10 w-10 place-items-center rounded-xl bg-primary text-primary-content">
        <svg viewBox="0 0 24 24" class="h-6 w-6" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" d="M4 7.5h16v11H4zM7 4.5h10v3H7zM8 11h8M8 14h5" />
        </svg>
      </div>
      <div>
        <div class="text-[15px] font-semibold tracking-tight">Scanserver</div>
        <div class="mt-0.5 text-xs text-base-content/60">{{ $t('navigation.scan') }}</div>
      </div>
    </div>

    <nav class="mt-8" aria-label="Main navigation">
      <ul class="menu w-full gap-1 p-0 text-sm">
        <li v-for="item in items" :key="item.path">
          <router-link :to="item.path" custom v-slot="{ href, navigate, isActive }">
            <a :href="href" :class="{ 'menu-active': isActive }" @click="navigate">
              <svg viewBox="0 0 24 24" class="h-[18px] w-[18px] shrink-0" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" :d="item.icon" />
              </svg>
              {{ item.label }}
            </a>
          </router-link>
        </li>
      </ul>
    </nav>

    <div class="mt-auto border-t border-base-300 px-3 pt-5 text-xs text-base-content/60">
      <div>{{ $t('navigation.version') }} {{ version }}</div>
      <div class="mt-1">&copy; {{ new Date().getFullYear() }} Scanserver</div>
    </div>
  </aside>

  <header class="sticky top-0 z-20 flex h-14 items-center border-b border-base-300 bg-base-100/95 px-4 backdrop-blur lg:hidden">
    <div class="flex items-center gap-2.5">
      <div class="grid h-8 w-8 place-items-center rounded-lg bg-primary text-primary-content">
        <svg viewBox="0 0 24 24" class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="M4 7.5h16v11H4zM7 4.5h10v3H7zM8 11h8M8 14h5" /></svg>
      </div>
      <span class="text-sm font-semibold tracking-tight">Scanserver</span>
    </div>
    <span class="ml-auto text-xs text-base-content/60">v{{ version }}</span>
  </header>

  <nav class="dock dock-sm z-30 border-t border-base-300 bg-base-100/95 lg:hidden" aria-label="Main navigation">
    <router-link v-for="item in items" :key="item.path" :to="item.path" custom v-slot="{ href, navigate, isActive }">
      <a :href="href" :aria-current="isActive ? 'page' : undefined" :class="isActive ? 'dock-active text-primary' : 'text-base-content/60'" @click="navigate">
        <svg viewBox="0 0 24 24" class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" :d="item.icon" /></svg>
        <span class="dock-label">{{ item.label }}</span>
      </a>
    </router-link>
  </nav>
</template>

<script>
import Constants from '../classes/constants';

const icons = {
  scan: 'M5 5h14v14H5z M8 9h8 M8 12h8 M8 15h5',
  files: 'M7 4h8l4 4v12H7z M15 4v5h4 M4 8v13h12 M10 12h5 M10 16h5',
  print: 'M6 8V3h12v5M6 17H4V9h16v8h-2M7 14h10v7H7zM17 11h.01',
  settings: 'M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7Z M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-1.7 2.9-.2-.1a1.7 1.7 0 0 0-1.8 0l-.2.1h-3.4l-.1-.2a1.7 1.7 0 0 0-1.5-1.1 1.7 1.7 0 0 0-1.6.8l-.1.2-3-1.7.1-.2a1.7 1.7 0 0 0 0-1.8l-.1-.2v-3.4l.2-.1a1.7 1.7 0 0 0 .3-3.1l-.2-.1 1.7-3 .2.1a1.7 1.7 0 0 0 1.8 0l.2-.1h3.4l.1.2a1.7 1.7 0 0 0 3.1.3l.1-.2 3 1.7-.1.2a1.7 1.7 0 0 0 0 1.8l.1.2v3.4l-.2.1a1.7 1.7 0 0 0-.6 1.5Z',
  about: 'M12 16v-4 M12 8h.01 M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z'
};

export default {
  name: 'Navigation',
  data() {
    return { version: Constants.Version };
  },
  computed: {
    items() {
      return [
        { path: '/scan', label: this.$t('navigation.scan'), icon: icons.scan },
        { path: '/files', label: this.$t('navigation.files'), icon: icons.files },
        { path: '/print', label: this.$t('navigation.print'), icon: icons.print },
        { path: '/settings', label: this.$t('navigation.settings'), icon: icons.settings },
        { path: '/about', label: this.$t('navigation.about'), icon: icons.about }
      ];
    }
  }
};
</script>
