// Outputs Test File
// Tests various output declarations and features

type customType = {
  id: string
  name: string
  location: string
}

// Simple outputs with decorators
@description('Simple string output')
output stringOutput string = 'Hello World'

@description('Simple integer output')
output intOutput int = 42

@description('Simple boolean output')
output boolOutput bool = true

// Secure output
@description('Secure string output')
@secure()
output secureStringOutput string = 'secure value'

// Output with value constraints
@description('Integer with min and max value constraints')
@minValue(0)
@maxValue(100)
output constrainedIntOutput int = 50

@description('String with length constraints')
@minLength(3)
@maxLength(20)
output constrainedStringOutput string = 'constrained'

// Array output
@description('Array output')
output arrayOutput array = [
  'item1'
  'item2'
  'item3'
]

// Object output
@description('Object output')
output objectOutput object = {
  name: 'test'
  enabled: true
  count: 42
}

// Output with custom type
@description('Output with custom type')
output customTypeOutput customType = {
  id: 'resource-id'
  name: 'resource-name'
  location: 'eastus'
}

// Output referencing a resource
@description('Output referencing a resource')
output resourceOutput string = resourceGroup().id

// Output using conditional expression
@description('Conditional output')
output conditionalOutput string = true ? 'value if true' : 'value if false'

// Output with complex expression
@description('Output with complex expression')
output complexOutput int = length([
  'item1'
  'item2'
  'item3'
])

// System namespace description
@sys.description('Output with system namespace description')
output sysDescribedOutput string = 'sys described value'
