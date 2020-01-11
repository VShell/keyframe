import { module, test } from 'qunit';
import { setupTest } from 'ember-qunit';

module('Unit | Route | stream', function(hooks) {
  setupTest(hooks);

  test('it exists', function(assert) {
    let route = this.owner.lookup('route:stream');
    assert.ok(route);
  });
});
