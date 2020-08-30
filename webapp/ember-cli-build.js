'use strict';

const EmberApp = require('ember-cli/lib/broccoli/ember-app');
const Project = require('ember-cli/lib/models/project');
const process = require('process');

module.exports = function(defaults) {
  let ENV = Project.closestSync(process.cwd()).config(EmberApp.env());

  let app = new EmberApp(defaults, {
    fingerprint: {
      prepend: ENV.assetRootURL,
      extensions: ['js', 'css', 'png', 'jpg', 'gif', 'map', 'svg', 'eot', 'ttf', 'woff', 'woff2'],
      exclude: ['assets/conversejs/locales']
    },

    'ember-bootstrap': {
      'bootstrapVersion': 4,
      'importBootstrapCSS': true
    }
  });

  // Use `app.import` to add additional libraries to the generated
  // output files.
  //
  // If you need to use different assets in different
  // environments, specify an object as the first parameter. That
  // object's keys should be the environment name and the values
  // should be the asset to use in that environment.
  //
  // If the library that you are including contains AMD or ES6
  // modules that you would like to import into your application
  // please specify an object with the list of modules as keys
  // along with the exports of each module as its value.
  app.import({
    development: 'vendor/dash.all.debug.js',
    production:  'vendor/dash.all.min.js'
  });
  app.import({
    development: 'vendor/converse.js',
    production:  'vendor/converse.min.js'
  });
  app.import({
    development: 'vendor/converse.css',
    production:  'vendor/converse.min.css'
  });
  app.import('vendor/emojis.js');

  return app.toTree();
};
