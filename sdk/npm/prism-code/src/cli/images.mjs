/** `prism-code images <verb>` (#442): the image verbs, under the module's name. */
export const verbs = Object.freeze({
  build: Object.freeze({ flat: 'build', usage: '...', summary: 'the same as prism-code build' }),
  inspect: Object.freeze({ flat: 'inspect', usage: '<app.holo> [--json]', summary: 'the same as prism-code inspect' }),
  pull: Object.freeze({ flat: 'pull', usage: '<reference> [<out.holo>]', summary: 'the same as prism-code pull' }),
  push: Object.freeze({ flat: 'push', usage: '<app.holo> <reference>', summary: 'the same as prism-code push' }),
  verify: Object.freeze({ flat: 'check', usage: '<app.holo> [--policy <file>] [--json]', summary: 'the same as prism-code check' }),
});
