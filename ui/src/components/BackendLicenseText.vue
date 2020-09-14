<template>
  <div>
    <h1>{{ $route.params.packageId }}</h1>

    <pre>{{ licenseText }}</pre>

    <b-link @click="$router.back()">Back</b-link>
  </div>
</template>

<script>
import { backendLicenses } from '!!val-loader!../licenses.js';

export default {
  name: 'BackendLicenseText',
  computed: {
    licenseText() {
      return backendLicenses[this.$route.params.packageId].licenseText;
    },
  },
};
</script>
