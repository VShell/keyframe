import Vue from 'vue';
import VueRouter from 'vue-router';

import Stream from './components/Stream.vue';
import About from './components/About.vue';
import License from './components/License.vue';
import LicenseText from './components/LicenseText.vue';
import BackendLicenseText from './components/BackendLicenseText.vue';

const routes = [
  { path: '/stream/:stream', component: Stream },
  { path: '/view/about', component: About },
  { path: '/view/license', component: License },
  { path: '/view/license/node-package/:packageId*', component: LicenseText },
  { path: '/view/license/backend/:packageId*', component: BackendLicenseText },
];

Vue.use(VueRouter);

const router = new VueRouter({
  mode: 'history',
  routes,
});

export default router;
