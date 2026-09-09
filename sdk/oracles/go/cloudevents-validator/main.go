// cloudevents-validator is a deliberately thin executable wrapper around the
// pinned official CloudEvents Go SDK. The SDK, rather than Prism code, owns
// JSON decoding and CloudEvents 1.0 semantic validation.
package main

import (
	"encoding/json"
	"fmt"
	"os"

	cloudevents "github.com/cloudevents/sdk-go/v2"
)

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "usage: cloudevents-validator INPUT")
		os.Exit(2)
	}
	input, err := os.ReadFile(os.Args[1])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}
	var event cloudevents.Event
	if err := json.Unmarshal(input, &event); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	if err := event.Validate(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
