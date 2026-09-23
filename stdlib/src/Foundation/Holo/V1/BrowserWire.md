# Browser surface wire profile

`BrowserWire.lex.tex` specializes the existing modeled physical-v4 codec for
the required View surface `prismpm-browser/1`. Its provenance extension key is
`https://uor.foundation/extension/prismpm-browser/v1`. Legacy `Wire` functions,
the `portable` surface and their exact output bytes remain unchanged.

The manifest retains the primary `hologram:guest/core-wasm@1` layer, View layer,
empty native-capability request and four content blobs. The new selector is
mandatory; unknown or downgraded selectors, foreign extension keys and mixed
profiles reject. Browser effects require their separately accepted Prism
policy and runtime. Empty native capabilities grant no browser authority;
zero native budgets must not be interpreted as restrictive limits.

This codec establishes structural framing only. Host verification must check
content digests, footer, directory, closed provenance and the complete accepted
application closure. Synthetic upstream vectors are not runnable applications
or evidence of browser/service acceptance. The unchanged pinned Live runtime
must reject the unfamiliar surface before preparing an execution session.
