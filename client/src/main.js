import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import Loader from "./components/Loader.vue";



//
// Bootstrap
//

import { BootstrapVue, IconsPlugin } from 'bootstrap-vue'

// Import Bootstrap an BootstrapVue CSS files (order is important)
import 'bootstrap/dist/css/bootstrap.css'
import 'bootstrap-vue/dist/bootstrap-vue.css'

// Make BootstrapVue available throughout your project
Vue.use(BootstrapVue)
// Optionally install the BootstrapVue icon components plugin
Vue.use(IconsPlugin)



//
// App
//

Vue.config.productionTip = false;

Vue.component('loader', Loader);

new Vue({
  router,
  render: h => h(App)
}).$mount("#app");
