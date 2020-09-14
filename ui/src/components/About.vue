<template>
  <div>
    <h1>About Keyframe</h1>

    <p><b-link @click="$router.back()">Back</b-link></p>

    <p>Keyframe is a free and open source live media streaming server.</p>

    <p><b-link to="/view/license">License information can be found here.</b-link></p>

    <p><b-link href="https://fossil.keyframe.live">The source code for Keyframe can be found here.</b-link></p>

    <p><b-link @click="$router.back()">Back</b-link></p>
  </div>
</template>

<script>
import licenses from '!!val-loader!../licenses.js';

export default {
  name: 'License',
  data() {
    return {
      licenses
    };
  },
  methods: {
    licenseLink(name) {
      return '/view/license/node-package/' + name;
    },
  },
};
</script>
