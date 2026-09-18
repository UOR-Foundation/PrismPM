export class ViewHostError extends Error {
  constructor(code, detail = null) {
    super(code); this.name = 'ViewHostError'; this.code = code; this.detail = detail;
  }
}
export const fail = code => new ViewHostError(code);
