// Parameters Test File
// Tests various parameter declarations and features

// Simple parameters with decorators
@description('This is a required string parameter')
@minLength(3)
@maxLength(20)
param requiredStringParam string

@description('This is an optional string parameter')
param optionalStringParam string?

@description('This is a secure string parameter')
@secure()
param secureStringParam string

// Parameters with default values
@description('Parameter with default string value')
param stringWithDefault string = 'default value'

@description('Parameter with default int value')
param intWithDefault int = 42

@description('Parameter with default bool value')
param boolWithDefault bool = true

// Object parameters
@description('Parameter with object type')
param simpleObject object = {
  name: 'test'
  enabled: true
  count: 42
}

// Object with specific structure
@description('Parameter with inline type definition')
param typedObject {
  @description('Name property')
  @minLength(3)
  name: string

  @description('Age property')
  @minValue(0)
  @maxValue(120)
  age: int

  @description('Optional email')
  email: string?

  @description('Nested object')
  address: {
    street: string
    city: string
    zipCode: string
  }
}

// Array parameters
@description('String array parameter')
param stringArray array = [
  'one'
  'two'
  'three'
]

@description('Typed array parameter')
param numberArray int[] = [1, 2, 3, 4, 5]

// Union type parameters
@description('Parameter with union type')
param unionParam 'Low' | 'Medium' | 'High' = 'Medium'

// Parameters with allowed values
@allowed([
  'Standard_LRS'
  'Standard_GRS'
  'Standard_ZRS'
  'Premium_LRS'
])
param storageSkuName string = 'Standard_LRS'

// Multi-line string parameter
@description('Parameter with multi-line string')
param multiLineParam string = '''
This is a multi-line string parameter.
It can contain multiple lines of text.
'''

// Parameter using a custom type
@description('Parameter using custom type')
param configParam {
  @minLength(5)
  prefix: string

  @minValue(1)
  @maxValue(100)
  instances: int
}

// Parameter with metadata decorator
@metadata({
  name: 'Complex Parameter'
  category: 'Configuration'
  version: '1.0'
  examples: [
    {
      name: 'example1'
      value: 'test'
    }
  ]
})
param complexParam object
