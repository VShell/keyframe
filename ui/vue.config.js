const path = require('path');
const escapeStringRegexp = require('escape-string-regexp');
const { ProvidePlugin } = require('webpack');

const assetsDir = 'stream-meta/ui/assets';
const converseAssetsDir = assetsDir + '/converse';

const config = {
  chainWebpack: config => {
    const converse = path.resolve(__dirname, 'converse/dist');
    const converseJs = path.resolve(converse, 'converse.js');
    config
      .resolve
        .symlinks(false)
        .alias
          .set('converse.js', path.resolve(__dirname, 'converse'))
          .end()
        .end()
      .module
        .rule('eslint')
          .exclude
            .add(converse)
            .end()
          .end()
        .end()
      .plugin('copy')
        .tap(([pathConfigs]) => {
          const extraPathConfigs = ['emojis.js', 'emojis.js.map', 'custom_emojis', 'locales'].map(name => {
            return {
              from: path.resolve(converse, name),
              to: path.resolve(pathConfigs[0].to, converseAssetsDir, name),
            };
          });
          return [[...pathConfigs, ...extraPathConfigs]];
        })
        .end()
      .plugin('define')
        .tap(([definitions]) => {
          Object.assign(definitions, {
            converseAssetsPath: JSON.stringify(converseAssetsDir),
          });
          return [definitions];
        })
        .end();
  },
  devServer: {
    proxy: {
      '^/(stream/[a-zA-Z0-9]+.mpd$|stream-meta)': {
        target: 'https://teststream.keyframe.alterednarrative.net',
        changeOrigin: true,
      },
    },
  },
  assetsDir,
};

module.exports = config;
