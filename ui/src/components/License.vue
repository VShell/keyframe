<template>
  <div>
    <h1>Licenses</h1>

    <b-link @click="$router.back()">Back</b-link>

    <h2>Backend licenses</h2>

    <table class="table">
      <thead>
        <tr>
          <th scope="col">Component</th>
          <th scope="col">License</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(license, name) in backendLicenses" :key="name">
          <td>{{ name }}</td>
          <td><router-link :to="backendLicenseLink(name)">{{ license.license }}</router-link></td>
        </tr>
      </tbody>
    </table>

    <h2>Frontend licenses</h2>

    <p>Note that not all packages below are distributed as part of the frontend - some are used as build tools.</p>

    <table class="table">
      <thead>
        <tr>
          <th scope="col">Package</th>
          <th scope="col">License</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(license, name) in licenses" :key="name">
          <td>{{ name }}</td>
          <td v-if="license.licenseText"><router-link :to="licenseLink(name)">{{ license.license }}</router-link></td>
          <td v-else>{{ license.license }}</td>
        </tr>
      </tbody>
    </table>

    <b-link @click="$router.back()">Back</b-link>
  </div>
</template>

<script>
import {licenses, backendLicenses} from '!!val-loader!../licenses.js';

export default {
  name: 'License',
  data() {
    return {
      licenses,
      backendLicenses
    };
  },
  methods: {
    licenseLink(name) {
      return '/view/license/node-package/' + name;
    },
    backendLicenseLink(name) {
      return '/view/license/backend/' + name;
    },
  },
};
</script>
