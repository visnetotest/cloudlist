package policy

import (
	"io/ioutil"

	"gopkg.in/yaml.v2"
)

// LoadPoliciesFromYAML loads a slice of policies from a YAML file.
// This function provides the default mechanism for loading policies from the local filesystem.
func LoadPoliciesFromYAML(filePath string) ([]Policy, error) {
	var policies []Policy

	// Read the YAML file content.
	data, err := ioutil.ReadFile(filePath)
	if err != nil {
		return nil, err
	}

	// Unmarshal the YAML into our slice of Policy structs.
	if err := yaml.Unmarshal(data, &policies); err != nil {
		return nil, err
	}

	return policies, nil
}
