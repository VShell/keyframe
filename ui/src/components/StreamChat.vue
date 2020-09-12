<script>
import converse from 'converse.js/dist/converse.min.js';
import styles from '!!file-loader?outputPath=stream-meta/ui/assets/css!extract-loader!css-loader!source-map-loader!converse.js/dist/converse.min.css';
import extraStyles from '!!file-loader?outputPath=stream-meta/ui/assets/css!extract-loader!css-loader!source-map-loader!./StreamChat.css';

converse.plugins.add('expose-closure', {
  initialize() {
    window._converse = this._converse;
  },
});

converse.plugins.add('autojoin', {
  initialize() {
    const _converse = this._converse;
    _converse.api.settings.update({
      autojoin: null
    });
    _converse.api.listen.on('presencesInitialized', () => {
      const room_jid = _converse.api.settings.get('autojoin');
      if (room_jid) {
        _converse.api.rooms.open(room_jid);
      }
    });
  },

  overrides: {
    ChatBox: {
      validate(attrs) {
        if (!attrs.jid) {
          return 'Ignored ChatBox without JID';
        }
      },
    },
  },
});

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

export default {
  name: 'StreamChat',
  props: {
    domain: {
      type: String,
      required: true,
    },
    stream: {
      type: String,
      required: true,
    },
  },
  render(h) {
    return h('div');
  },
  created() {
    this.shadow = null;
    this.fontFaces = null;
  },
  mounted() {
    const shadow = this.shadow = this.$el.attachShadow({ mode: 'open' });
    const converseAssetsUrl = process.env.BASE_URL + converseAssetsPath + (converseAssetsPath.endsWith('/') ? '' : '/');

    let style = document.createElement('link');
    style.rel = 'stylesheet';
    style.href = styles;
    style.addEventListener('load', () => {
      let newRules = '';
      for(const rule of style.sheet.cssRules) {
        if (rule.type == CSSRule.FONT_FACE_RULE) {
          newRules += rule.cssText + '\n';
        }
      }

      if (!this.fontFaces) {
        const fontFaces = this.fontFaces = document.createElement('style');
        document.head.appendChild(fontFaces);
      }

      this.fontFaces.innerText = newRules;
    });
    shadow.appendChild(style);
    let extraStyle = document.createElement('link');
    extraStyle.rel = 'stylesheet';
    extraStyle.href = extraStyles;
    shadow.appendChild(extraStyle);

    const room_jid = this.stream + '@streamchat.' + this.domain;

    converse.initialize({
      loglevel: 'debug',
      root: shadow,
      assets_path: converseAssetsUrl,
      bosh_service_url: '/stream-meta/bosh',
      view_mode: 'embedded',
      singleton: true,
      whitelisted_plugins: [
        'expose-closure',
        'autojoin',
        'hide-chatroom-participants',
      ],
      authentication: 'anonymous',
      auto_login: true,
      allow_logout: false,
      jid: 'streamguest.' + this.domain,
      /*auto_join_rooms: [
        room_jid,
      ],*/
      autojoin: room_jid,
      hide_muc_server: true,
      allow_muc_invitations: false,
    });
  },
};
</script>
