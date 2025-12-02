package policy

import "github.com/projectdiscovery/cloudlist/pkg/schema"

// Policy represents a single, named policy containing a set of rules.
type Policy struct {
	ID          string `yaml:"id"`
	Name        string `yaml:"name"`
	Description string `yaml:"description"`
	Rules       []Rule `yaml:"rules"`
}

// Rule defines a single condition to be evaluated against a resource.
// A rule is a simple key-value match on a resource's properties.
type Rule struct {
	// Field is the name of the field in the Resource struct to check (e.g., "Provider").
	Field string `yaml:"field"`
	// Value is the expected value for the field.
	Value string `yaml:"value"`
}

// Violation represents a single instance of a policy rule being broken by a resource.
type Violation struct {
	PolicyID     string
	PolicyName   string
	Asset        *schema.Resource
	ViolatedRule Rule
	Details      string
}
