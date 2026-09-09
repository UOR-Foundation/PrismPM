// intoto-statement-validator is a deliberately narrow executable wrapper
// around the generated Go protobuf published by the in-toto Attestation
// project. It validates only the JSON/protobuf wire shape owned by that
// upstream schema. The upstream proto explicitly leaves field semantics to
// consumers, so this executable must not be treated as policy validation.
package main

import (
	"fmt"
	"os"

	statement "github.com/in-toto/attestation/go/spec/v1.0"
	"google.golang.org/protobuf/encoding/protojson"
)

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "usage: intoto-statement-validator INPUT")
		os.Exit(2)
	}
	input, err := os.ReadFile(os.Args[1])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}
	var value statement.Statement
	if err := (protojson.UnmarshalOptions{DiscardUnknown: false}).Unmarshal(input, &value); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	if _, err := (protojson.MarshalOptions{UseProtoNames: false}).Marshal(&value); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
