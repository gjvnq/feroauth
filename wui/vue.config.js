module.exports = {
  pluginOptions: {
    i18n: {
      locale: 'en',
      fallbackLocale: 'en',
      localeDir: 'locales',
      enableInSFC: true
    }
  },

  assetsDir: 'static',

  css: {
    sourceMap: true
  },

  pages: {
    login: {
      entry: 'src/main-login.js',
      template: 'public/login.html',
      filename: 'login.html'
    },
    index: {
      entry: 'src/main.js',
      template: 'public/index.html',
      filename: 'index.html',
    } 
  }
}
