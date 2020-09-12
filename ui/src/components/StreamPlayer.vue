<template>
  <div>
    <div :class="$style.videoContainer">
      <video v-once ref="player" :class="$style.video"></video>
    </div>
    <ControlBar :player="player" v-if="initialized" />
  </div>
</template>

<script>
import { MediaPlayer, Debug } from 'dashjs';
import ControlBar from './controlbar/ControlBar.vue';

export default {
  name: 'StreamPlayer',
  props: {
    mpdUrl: {
      type: String,
      required: true,
    },
  },
  data() {
    return {
      initialized: false,
    };
  },
  mounted() {
    const player = this.player = MediaPlayer().create();

    player.clearDefaultUTCTimingSources();
    player.updateSettings({
      streaming: {
        lowLatencyEnabled: true,
        useManifestDateHeaderTimeSource: true,
        liveDelay: 3,
        liveCatchUpMinDrift: 0.2,
        liveCatchupPlaybackRate: 0.05,
      },
      debug: {
        logLevel: Debug.LOG_LEVEL_DEBUG,
      },
    });

    player.on(MediaPlayer.events.PLAYBACK_NOT_ALLOWED, () => {
      console.log('Playback did not start due to auto play restrictions. Muting audio and reloading');
      this.$refs.player.muted = true;
      this.player.seek(this.player.duration()-4);
      this.player.play();
    });

    player.initialize(this.$refs.player, this.mpdUrl, true);
    this.initialized = true;
  },
  beforeDestroy() {
    if (this.player) {
      this.player.reset();
    }
  },
  methods: {
    seekToLive() {
      this.player.seek(this.player.duration()-4);
    },
  },
  components: {
    ControlBar,
  },
};
</script>

<style module>
.videoContainer {
  position: relative;
  width: 100%;
  padding-top: calc(100% * (9 / 16));
}

.video {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

/*.controlbarContainer {
  display: none;
  position: absolute;
  left: 0px;
  bottom: 0px;
  width: 100%;
}

.container:hover .controlbarContainer {
  display: block;
}*/
</style>
