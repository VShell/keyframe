<template>
  <div class="video-controller unselectable">
    <div v-on:click="playToggle" class="btn-play-pause" title="Play/Pause">
      <span :class="playingIcon"></span>
    </div>
    <div v-on:click="muteToggle" class="btn-mute control-icon-layout" title="Mute">
      <span :class="muteIcon"></span>
    </div>
    <div>
      <input type="range" v-model="volume" class="volumebar" value="1" min="0" max="1" step=".01"/>
    </div>
    <div class="time-display">{{ timeText }}</div>
    <div class="seekContainer">
      <div ref="seekbar" v-on:mousedown="seekStart" class="seekbar seekbar-complete">
        <div class="seekbar seekbar-buffer"></div>
        <div class="seekbar seekbar-play" :style="seekPlayWidth"></div>
      </div>
    </div>
    <div v-on:click="seekLive" :class="['duration-display', 'live-icon', liveClass]">● LIVE</div>
  </div>
</template>

<script>
// This file contains code under the following license, in addition to code under the GPLv3 license
/**
 * The copyright in this software is being made available under the BSD License,
 * included below. This software may be subject to other third party and contributor
 * rights, including patent rights, and no such rights are granted under this license.
 *
 * Copyright (c) 2013, Dash Industry Forum.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *  * Redistributions of source code must retain the above copyright notice, this
 *  list of conditions and the following disclaimer.
 *  * Redistributions in binary form must reproduce the above copyright notice,
 *  this list of conditions and the following disclaimer in the documentation and/or
 *  other materials provided with the distribution.
 *  * Neither the name of Dash Industry Forum nor the names of its
 *  contributors may be used to endorse or promote products derived from this software
 *  without specific prior written permission.
 *
 *  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS AS IS AND ANY
 *  EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 *  WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 *  IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT,
 *  INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
 *  NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 *  PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 *  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 *  ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 *  POSSIBILITY OF SUCH DAMAGE.
 */

import { MediaPlayer } from 'dashjs';

const liveThresholdSecs = 4;

export default {
  name: 'VideoControlBar',
  props: {
    player: {
      type: Object,
      required: true,
    },
  },
  data() {
    return {
      playing: false,
      volume: 1,
      mute: false,
      time: 0,
      duration: 0,
      seeking: false,
      captionTracks: [],
    };
  },
  mounted() {
    const player = this.player;
    player.on(MediaPlayer.events.PLAYBACK_STARTED, this.dashPlayStart, this);
    player.on(MediaPlayer.events.PLAYBACK_PAUSED, this.dashPlayPaused, this);
    player.on(MediaPlayer.events.PLAYBACK_TIME_UPDATED, this.dashPlayTimeUpdated, this);
    player.on(MediaPlayer.events.STREAM_INITIALIZED, this.dashStreamInitialized, this);
    player.on(MediaPlayer.events.STREAM_TEARDOWN_COMPLETE, this.dashStreamTeardownComplete, this);
    player.getVideoElement().addEventListener('volumechange', this.videoVolumeChanged);

    this.playing = !player.isPaused();
    this.volume = player.getVolume();
    this.mute = player.isMuted();
  },
  beforeDestroy() {
    const player = this.player;
    player.off(MediaPlayer.events.PLAYBACK_STARTED, this.dashPlayStart, this);
    player.off(MediaPlayer.events.PLAYBACK_PAUSED, this.dashPlayPaused, this);
    player.off(MediaPlayer.events.PLAYBACK_TIME_UPDATED, this.dashPlayTimeUpdate, this);
    player.off(MediaPlayer.events.STREAM_INITIALIZED, this.dashStreamInitialized, this);
    player.off(MediaPlayer.events.STREAM_TEARDOWN_COMPLETE, this.dashStreamTeardownComplete, this);
    player.getVideoElement().removeEventListener('volumechange', this.videoVolumeChanged);
  },
  computed: {
    playingIcon() {
      return [this.playing ? "icon-pause" : "icon-play"];
    },
    muteIcon() {
      return [this.mute ? "icon-mute-on" : "icon-mute-off"];
    },
    timeText() {
      const liveDelay = this.duration - this.time;
      if (isNaN(liveDelay) || liveDelay < liveThresholdSecs) {
        return '';
      } else {
        return '- ' + this.player.convertToTimeCode(liveDelay);
      }
    },
    seekPlayWidth() {
      if (isNaN(this.duration)) {
        return {};
      }
      const liveDelay = this.duration - this.time;
      return {
        width: (!this.seeking && liveDelay < liveThresholdSecs) ? '100%' : (this.time / this.duration * 100) + '%',
      };
    },
    liveClass() {
      if (isNaN(this.duration)) {
        return null;
      }
      const liveDelay = this.duration - this.time;
      return liveDelay < liveThresholdSecs ? "live" : null;
    },
  },
  watch: {
    volume(newVolume) {
      const player = this.player;
      player.setVolume(parseFloat(newVolume));
      this.mute = player.getVolume() == 0;
      this.player.setMute(this.mute);
    },
  },
  methods: {
    playToggle() {
      this.playing = !this.playing;
      if (this.playing) {
        this.player.play();
      } else {
        this.player.pause();
      }
    },
    muteToggle() {
      this.mute = !this.mute;
      this.player.setMute(this.mute);
    },
    seekLive() {
      console.log("seeking to live");
      this.player.seek(this.player.duration());
    },
    seekStart(evt) {
      this.seeking = true;
      this.dashUpdateTimeByEvent(evt);
      document.addEventListener('mousemove', this.seekMouseMove, true);
      document.addEventListener('mouseup', this.seekEnd, true);
    },
    seekMouseMove(evt) {
      this.dashUpdateTimeByEvent(evt);
    },
    seekEnd(evt) {
      this.seeking = false;
      console.log("seek end", evt);
      if(!isNaN(this.dashUpdateTimeByEvent(evt))) {
        console.log("seeking");
        this.player.seek(this.time);
      }
      document.removeEventListener('mousemove', this.seekMouseMove, true);
      document.removeEventListener('mouseup', this.seekEnd, true);
    },
    dashUpdateTimeByEvent(evt) {
      const seekbarRect = this.$refs.seekbar.getBoundingClientRect();
      const time = Math.floor(this.player.duration() * (evt.clientX - seekbarRect.left) / seekbarRect.width);
      if (!isNaN(time)) {
        this.time = time;
      }
      return time;
    },
    dashPlayStart() {
      this.playing = true;
      this.time = this.player.time();
      this.dashUpdateDuration();
    },
    dashPlayPaused() {
      this.playing = false;
    },
    dashPlayTimeUpdated() {
      this.dashUpdateDuration();
      if (!this.seeking) {
        this.time = this.player.time();
      }
    },
    dashStreamInitialized() {
      this.dashUpdateDuration();
    },
    dashStreamTeardownComplete() {
      this.playing = false;
      this.time = 0;
    },
    dashUpdateDuration() {
      this.duration = this.player.duration();
    },
    videoVolumeChanged() {
      if (this.volume != this.player.getVolume()) {
        this.volume = this.player.getVolume();
      }
      this.mute = this.player.isMuted();
    },
  },
};
</script>

<style scoped>
.unselectable {
    -webkit-touch-callout: none;
    -webkit-user-select: none;
    -khtml-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
}

.time-display,
.duration-display{
    padding:11px;
    color: white;
    font-weight: normal;
    font-size: .9em;
    font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
}

.time-display {
}

.duration-display{
}

.live-icon {
    cursor: pointer;
}

.live-icon.live {
    color:red !important;
    pointer-events: none;
    cursor: default;
}

.btn-play-pause {
    padding:9px 10px;
    cursor: pointer;
}

.control-icon-layout {
    padding: 9px 10px;
    cursor: pointer;
}

.btn-fullscreen {
    margin-right: 10px;
}

.volumebar {
    width: 70px;
}

.video-controller {
    min-height:35px;
    z-index: 2147483646;
    display: flex;
    align-items: center;
}

.video-controller-fullscreen {
    position: fixed;
    z-index:2147483647;
    width: 100%;
    bottom: 0;
    left: 0;
}

.menu,
.video-controller {
    background-color: black;
}

.menu-item-unselected,
.menu-item-selected{
    font-weight: normal;
    font-size: .9em;
    font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
}

.menu-item-unselected {
    color: white;
}

.menu-item-over,
.menu-item-selected {
    background-color: white;
    color: black;
}

.menu-sub-menu-title {
    background-color: #191919;
    padding-left: 2px;
    font-weight: bold;
    font-size: 1.0em;
    font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;

}

.menu-item-selected {
    opacity: .7;
}

.menu ul{
    list-style-type: none;
    padding:0;
    margin:0;
}

.menu li{
    padding:0 10px;
    cursor: pointer;
}

.menu {
    position: absolute;
}

#bitrateMenu .menu-sub-menu-title {
    min-width: 150px;
}

@font-face {
    font-family: 'icomoon';
    src: url("icomoon.ttf") format("truetype");
    font-weight: normal;
    font-style: normal;
}

.icon-play,
.icon-pause,
.icon-caption,
.icon-mute-off,
.icon-mute-on,
.icon-fullscreen-enter,
.icon-fullscreen-exit,
.icon-tracks,
.icon-bitrate {
    font-family: 'icomoon';
    font-size: 20px;
    color: white;
    text-shadow: none;
    -webkit-font-smoothing: antialiased;
}

.icon-fullscreen-enter:before {
    content: "\e90b";
}
.icon-fullscreen-exit:before {
    content: "\e90c";
}
.icon-play:before {
    content: "\e910";
}
.icon-pause:before {
    content: "\e911";
}
.icon-mute-on:before {
    content: "\e909";
}
.icon-mute-off:before {
    content: "\e918";
}
.icon-caption:before {
    content: "\e901";
}
.icon-bitrate:before {
    content: "\e905";
}
.icon-tracks:before {
    content: "\e90a";
}

.seekContainer {
    flex: 1;
    display: flex;
    overflow: auto;
    overflow-y: hidden;
    overflow-x: hidden;
}

input[type="range"] {
    -webkit-appearance: none;
    -webkit-tap-highlight-color: rgba(255, 255, 255, 0);
    height: 14px;
    border: none;
    margin:12px 5px;
    padding: 1px 2px;
    border-radius: 5px;
    background: #232528;
    box-shadow: inset 0 1px 0 0 #0d0e0f, inset 0 -1px 0 0 #3a3d42;
    -webkit-box-shadow: inset 0 1px 0 0 #0d0e0f, inset 0 -1px 0 0 #3a3d42;
    outline: none; /* no focus outline */
}

input[type=range]::-moz-focus-outer {
    border: 0;
}

input[type="range"]::-moz-range-track {
    border: inherit;
    background: transparent;
}

input[type="range"]::-ms-track {
    border: inherit;
    color: transparent; /* don't drawn vertical reference line */
    background: transparent;
}

input[type="range"]::-ms-fill-lower,
input[type="range"]::-ms-fill-upper {
    background: transparent;
}

input[type="range"]::-ms-tooltip {
    display: none;
}

/* thumb */
input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 15px;
    height: 8px;
    border: none;
    border-radius: 2px;
    background-color:rgb(0, 150, 215);
}
input[type="range"]::-moz-range-thumb {
    width: 15px;
    height: 8px;
    border: none;
    border-radius: 2px;
    background-color:rgb(0, 150, 215);
}

input[type="range"]::-ms-thumb {
    width: 15px;
    height: 8px;
    border: none;
    border-radius: 2px;
    background-color:rgb(0, 150, 215);
}

.thumbnail-container {
    position: absolute;
    text-align: center;
}

.thumbnail-elem {
    position: relative;
    box-shadow: 0px 0px 0.9em #000000;
    transform-origin: center bottom;
}

.thumbnail-time-label {
    position: relative;
    bottom: 1.8em;
    display: table;
    margin: 0 auto;
    padding: 2px 5px 2px 5px;
    color: #ffffff;
    background-color: rgba(0, 0, 0, 0.7);
    font-size: 12px;
    font-weight: bold;
    font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
}

.seekbar-complete {
    width: 100%;
    height: 7px;
    background: #999a99;
    position: relative;
    overflow: hidden;
}

.seekbar-buffer {
    position: absolute;
    left: 0px;
    top: 0px;
    width: 0%;
    height: 7px;
    background: lightgray;
}

.seekbar-play {
    position: absolute;
    left: 0px;
    top: 0px;
    width: 0%;
    height: 7px;
    background: rgb(0, 150, 215);
}
</style>
