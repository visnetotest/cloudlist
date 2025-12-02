package policy

import (
	"fmt"
	"reflect"

	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// NativeEvaluator is responsible for evaluating native policies against assets.
type NativeEvaluator struct {
}

// NewNativeEvaluator creates a new NativeEvaluator.
func NewNativeEvaluator() *NativeEvaluator {
	return &NativeEvaluator{}
}

// Evaluate runs the policy evaluation process.
// It takes a list of policies and a list of resources and returns any violations.
func (e *NativeEvaluator) Evaluate(policies []Policy, resources []*schema.Resource) ([]Violation, error) {
	var violations []Violation

	for _, resource := range resources {
		// Use reflection to inspect the resource's fields.
		val := reflect.ValueOf(resource).Elem()

		for _, policy := range policies {
			for _, rule := range policy.Rules {
				fieldValue := val.FieldByName(rule.Field)

				// Check if the field is valid and if its value matches the rule.
				if fieldValue.IsValid() {
					// Use fmt.Sprintf for a more robust string conversion for different field types.
					if fmt.Sprintf("%v", fieldValue.Interface()) == rule.Value {
						violation := Violation{
							PolicyID:     policy.ID,
							PolicyName:   policy.Name,
							Asset:        resource,
							ViolatedRule: rule,
							Details:      fmt.Sprintf("Asset field '%s' with value '%s' matched violation rule.", rule.Field, rule.Value),
						}
						violations = append(violations, violation)
					}
				}
			}
		}
	}

	return violations, nil
}
