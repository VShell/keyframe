import Route from '@ember/routing/route';

export default class StreamRoute extends Route {
  model(params) {
    let id = params.stream_id;
    return { id };
  }
}
