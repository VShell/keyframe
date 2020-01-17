import Component from '@glimmer/component';
import { action } from '@ember/object';
import ENV from 'keyframe/config/environment';

converse.plugins.add('hide-chatroom-participants', {
  dependencies: ['converse-muc-views'],
  overrides: {
    ChatRoomView: {
      initialize: function() {
        if(!this.model.has('hidden_occupants')) {
          this.model.save({'hidden_occupants': true});
        }
        this.__super__.initialize.apply(this, arguments);
      },
      renderAfterTransition: function() {
        this.__super__.renderAfterTransition.apply(this, arguments);
        if(this.model.get('hidden_occupants')) {
          converse.env.utils.hideElement(this.el.querySelector('.occupants'));
          this.scrollDown();
        }
      },
    },
  },
});

export default class StreamChatComponent extends Component {
  @action
  initConverseJS() {
    let domain = location.hostname;
    let room_jid = this.args.stream + '@streamchat.' + domain;

    converse.initialize({
      assets_path: ENV.assetRootURL+'assets/conversejs/',
      bosh_service_url: '/stream-meta/bosh',
      view_mode: 'embedded',
      singleton: true,
      whitelisted_plugins: [
        'hide-chatroom-participants',
      ],
      authentication: 'anonymous',
      auto_login: true,
      allow_logout: false,
      jid: 'streamguest.' + domain,
      auto_join_rooms: [
        room_jid,
      ],
      notify_all_room_messages: [
        room_jid,
      ],
      hide_muc_server: true,
      allow_muc_invitations: false,
    });
  }
}
