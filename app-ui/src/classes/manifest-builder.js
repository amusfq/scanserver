export default class ManifestBuilder {
  constructor() {
  }

  static create() {
    return new ManifestBuilder();
  }

  withDark(dark) {
    this.dark = dark;
    return this;
  }

  withStorage(storage) {
    this.storage = storage;
    return this;
  }

  themeColor() {
    return this.dark ? '#171d1a' : '#f5f6f3';
  }

  build() {
    return {
      theme_color : this.themeColor(),
      background_color : this.dark ? '#000000' : '#FFFFFF',
      display : 'standalone',
      scope : '/',
      start_url : '/scan',
      name : 'Scanserver',
      short_name : 'Scanserver',
      description : 'A simple network scanner',
      icons : [
        {
          src : './icons/android-chrome-192x192.png',
          sizes : '192x192',
          type : 'image/png',
          purpose : 'any'
        },
        {
          src : './icons/android-chrome-512x512.png',
          sizes : '512x512',
          type : 'image/png',
          purpose : 'any'
        },
        {
          src : './icons/android-chrome-maskable-192x192.png',
          sizes : '192x192',
          type : 'image/png',
          purpose : 'maskable'
        },
        {
          src : './icons/android-chrome-maskable-512x512.png',
          sizes : '512x512',
          type : 'image/png',
          purpose : 'maskable'
        }
      ]
    };
  }
}
