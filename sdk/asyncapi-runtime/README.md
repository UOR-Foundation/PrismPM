# AsyncAPI embedded-example runtime

The upstream `b3fac5bb` source, script, manifest and historical lock remain unchanged.
This Prism-owned lock refreshes their compatible dependencies while retaining
parser 3.6.0. It is not an upstream-reviewed lock or historical-lock replay.
Installation uses `npm ci --ignore-scripts`; no lifecycle hook is authorized.

The inventoried shell rejects ambient Node preloads and checks its fixed
JavaScript digest before startup. The launcher requires the fixed SDK inventory
and verifies the actual installed runtime byte tree before executing the untouched
script. Source/layout/version
checks alone do not authenticate package code. Tests use explicit isolated
pristine fixtures, never a production inventory fallback.

Historical-lock findings remain evidence. A clean installed-graph audit does not
establish a clean SDK image; complete image scanning and disposition remain required.
