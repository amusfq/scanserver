import { createApp } from 'vue';
import { createRouter, createWebHashHistory } from 'vue-router';
import { createI18n } from 'vue-i18n';
import messages from '@intlify/unplugin-vue-i18n/messages';
import '@/styles/main.css';

import Constants from './classes/constants';
import App from './App.vue';
import Files from './components/Files.vue';
import Scan from './components/Scan.vue';
import Settings from './components/Settings.vue';
import About from './components/About.vue';

const datetimeFormats = {};
for (const locale of Constants.Locales) {
  datetimeFormats[locale] = Constants.DateTimeFormat;
}

const i18n = createI18n({
  legacy: false,
  datetimeFormats,
  locale: import.meta.env.VITE_APP_I18N_LOCALE,
  fallbackLocale: import.meta.env.VITE_APP_I18N_FALLBACK_LOCALE,
  messages: messages,
  missingWarn: false,
  fallbackWarn: false
});

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/about', component: About },
    { path: '/files', component: Files },
    { path: '/settings', component: Settings },
    { path: '/scan', component: Scan },
    { path: '/', component: Scan }
  ]
});

createApp(App)
  .use(router)
  .use(i18n)
  .mount('#app');
