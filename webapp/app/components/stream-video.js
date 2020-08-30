import Component from '@glimmer/component';
import { action } from '@ember/object';

export default class StreamVideoComponent extends Component {
  @action
  initDash(element) {
    let player = dashjs.MediaPlayer().create();
    player.initialize(element, null, true);
    player.updateSettings({ 'streaming': { 'lowLatencyEnabled': true }});
    player.clearDefaultUTCTimingSources();
    player.attachSource('/stream/'+this.args.stream+'.mpd');
  }
}
