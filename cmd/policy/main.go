package main

import (
	"log"

	"github.com/projectdiscovery/cloudlist/pkg/policy"
	"github.com/projectdiscovery/goflags"
	"github.com/projectdiscovery/gologger"
)

func main() {
	options := &policy.Options{}
	flagSet := goflags.NewFlagSet()
	flagSet.StringVar(&options.PolicyPath, "policy", "", "Path to policy file or directory")
	flagSet.StringVarP(&options.InputFile, "input", "i", "", "Path to input file")
	flagSet.StringVarP(&options.Output, "output", "o", "", "Path to output file")
	flagSet.BoolVar(&options.Block, "block", false, "Block on policy failure")
	flagSet.StringSliceVarP(&options.Severity, "severity", "s", nil, "Severities to filter on (low, medium, high, critical)", goflags.NormalizedStringSliceOptions)

	if err := flagSet.Parse(); err != nil {
		log.Fatalf("Could not parse flags: %s", err)
	}

	policyCommand, err := policy.New(options)
	if err != nil {
		log.Fatalf("Could not create policy command: %s", err)
	}

	if err := policyCommand.Run(); err != nil {
		log.Fatalf("Could not run policy command: %s", err)
	}
}
