import Vue from 'vue'
import AppLogin from './AppLogin.vue'
import router from './router-login'
import store from './store'
import i18n from './i18n'
import { BootstrapVue, IconsPlugin } from 'bootstrap-vue'

// Import Bootstrap an BootstrapVue CSS files (order is important)
import 'bootstrap/dist/css/bootstrap.css'
import 'bootstrap-vue/dist/bootstrap-vue.css'

// Make BootstrapVue available throughout your project
Vue.use(BootstrapVue)
Vue.use(IconsPlugin)

Vue.config.productionTip = false

new Vue({
  router,
  store,
  i18n,
  render: h => h(AppLogin)
}).$mount('#app')
