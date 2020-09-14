<template>
  <div>
    <h1>{{ $route.params.packageId }}</h1>

    <pre>{{ licenseText }}</pre>

    <b-link @click="$router.back()">Back</b-link>
  </div>
</template>

<script>
import { licenses } from '!!val-loader!../licenses.js';

export default {
  name: 'LicenseText',
  computed: {
    licenseText() {
      return licenses[this.$route.params.packageId].licenseText;
    },
  },
};
</script>
