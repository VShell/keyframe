import Route from '@ember/routing/route';
import fetch from 'fetch';

export default class LicenseRoute extends Route {
  async model() {
    let dependencies = await fetch('/assets/licenses.json');
    dependencies = await dependencies.json();
    return { dependencies };
  }
}
