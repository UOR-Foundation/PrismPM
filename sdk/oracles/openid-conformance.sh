#!/bin/sh
set -eu

export JAVA_HOME=/opt/prismpm/openid-java
export PATH="$JAVA_HOME/bin:$PATH"

selection='ValidateIdTokenSignature_UnitTest,OIDCCValidateRequestObjectExp_UnitTest,CheckDiscEndpointIssuer_UnitTest,CheckDiscEndpointIssuerIsValidUrl_UnitTest'
if [ "$#" -gt 0 ] && [ "$1" != '--official-selection' ]; then
    selection=$1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
cp -a /opt/prismpm/share/standards/oracles/openid-conformance-suite-3e09b13b/. "$work/"
cd "$work"
cp -a /opt/prismpm/openid-target ./target
rm -rf target/surefire-reports
/opt/prismpm/openid-maven/bin/mvn \
    --offline \
    --quiet \
    -Dmaven.repo.local=/opt/prismpm/openid-m2 \
    -Dmaven.gitcommitid.skip=true \
    -Djacoco.skip=true \
    -Dtest="$selection" \
    surefire:test

if [ "$selection" = 'ValidateIdTokenSignature_UnitTest,OIDCCValidateRequestObjectExp_UnitTest,CheckDiscEndpointIssuer_UnitTest,CheckDiscEndpointIssuerIsValidUrl_UnitTest' ]; then
    set -- \
        target/surefire-reports/TEST-net.openid.conformance.condition.client.ValidateIdTokenSignature_UnitTest.xml \
        target/surefire-reports/TEST-net.openid.conformance.condition.as.OIDCCValidateRequestObjectExp_UnitTest.xml \
        target/surefire-reports/TEST-net.openid.conformance.condition.client.CheckDiscEndpointIssuer_UnitTest.xml \
        target/surefire-reports/TEST-net.openid.conformance.condition.client.CheckDiscEndpointIssuerIsValidUrl_UnitTest.xml
    [ "$#" -eq 4 ]
    for report in "$@"; do
        [ -f "$report" ]
        grep -Eq '<testsuite .*errors="0"' "$report"
        grep -Eq '<testsuite .*failures="0"' "$report"
        grep -Eq '<testsuite .*skipped="0"' "$report"
    done
    tests=$(sed -n 's/.*<testsuite .* tests="\([0-9][0-9]*\)".*/\1/p' "$@" | awk '{ total += $1 } END { print total + 0 }')
    [ "$tests" -eq 29 ]
    printf '%s\n' '{"errors":0,"failures":0,"officialTests":29,"skipped":0}'
fi
