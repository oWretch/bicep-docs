// Variables demonstration file
// Shows different types of variables and decorators

// Simple variable
var simpleVar = 'simple value'

// Variable with description
@description('A variable with a description')
var describedVar = 'described value'

// Variable with sys namespace description
@sys.description('A variable with a sys namespace description')
var sysDescribedVar = 'sys described value'

// Exported variable
@description('An exported variable')
@export()
var exportedVar = 'exported value'

// Number variable
var numberVar = 42

// Boolean variable
var boolVar = true

// Array variable
var arrayVar = [
  'item1',
  'item2',
  'item3'
]

// Object variable
var objectVar = {
  prop1: 'value1'
  prop2: 'value2'
  nestedObject: {
    nestedProp: 'nested value'
  }
}

// Variable using expressions
var expressionVar = 'prefix-${simpleVar}-suffix'

// Variable with multiple decorators
@description('Variable with multiple decorators')
@metadata({
  category: 'demo'
  priority: 'high'
})
var multiDecoratorVar = 'multi decorator value'
