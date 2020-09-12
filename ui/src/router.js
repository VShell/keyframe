import Vue from 'vue';
import VueRouter from 'vue-router';

import Stream from './components/Stream.vue';

const routes = [
  { path: '/stream/:stream', component: Stream },
];

Vue.use(VueRouter);

const router = new VueRouter({
  mode: 'history',
  routes,
});

export default router;
