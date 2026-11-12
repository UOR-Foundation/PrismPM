/** `prism-code containers <verb>` (#443): the container verbs, under the module's name. */
export const verbs = Object.freeze({
  inspect: Object.freeze({ flat: 'inspect', usage: '<container> [--json]', summary: 'the same as prism-code inspect' }),
  rm: Object.freeze({ flat: 'rm', usage: '<container>...', summary: 'the same as prism-code rm' }),
  run: Object.freeze({ flat: 'run', usage: '<app.holo> <input> ...', summary: 'the same as prism-code run' }),
});
