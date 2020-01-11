import Component from '@glimmer/component';
import { action } from '@ember/object';
import videojs from 'video.js';

export default class StreamVideoComponent extends Component {
  @action
  initVideoJs(element) {
    window.HELP_IMPROVE_VIDEOJS = false;

    let player = videojs(element, {
      controls: true,
      fluid: true,
      liveui: true,
      sources: [{
        src: '/stream/'+this.args.stream+'.m3u8',
        type: 'application/x-mpegURL',
        handleManifestRedirects: true,
      }],
    });

    player.ready(function() {
      player.play();
    });
  }
}
