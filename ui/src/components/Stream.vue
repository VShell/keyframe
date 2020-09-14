<template>
  <b-container fluid class="h-100 p-0">
    <b-row no-gutters class="h-100">
      <b-col cols="8" class="pl-1 pt-1">
        <StreamPlayer :mpdUrl="mpdUrl" />
      </b-col>
      <b-col cols="4" :class="[$style.streamchat, 'pl-3', 'pr-1', 'pt-1']">
        <StreamChat :stream="$route.params.stream" :domain="domain" class="h-100" />
      </b-col>
    </b-row>
  </b-container>
</template>

<script>
import StreamPlayer from './StreamPlayer.vue';
import StreamChat from './StreamChat.vue';

export default {
  name: 'Stream',
  computed: {
    mpdUrl() {
      return this.$route.path + '.mpd';
    },
    domain() {
      return 'teststream.keyframe.alterednarrative.net';
    },
  },
  components: {
    StreamPlayer,
    StreamChat,
  },
};
</script>

<style module>
.streamchat {
  height: calc(100vh - 3.5rem);
}
</style>
