// Decorators Test File
// Tests various decorators in Bicep

// Description decorators
@description('Description decorator example')
param descriptionParam string

@sys.description('System namespace description decorator')
param sysDescriptionParam string

// Constraint decorators for string parameters
@minLength(3)
@maxLength(20)
param stringConstraintParam string

// Constraint decorators for numeric parameters
@minValue(0)
@maxValue(100)
param numericConstraintParam int

// Allowed values decorator
@allowed([
  'option1'
  'option2'
  'option3'
])
param allowedValuesParam string

// Secure decorator
@secure()
param secureParam string

// Export decorator
@export()
param exportParam string

@export()
type exportedType = {
  name: string
  value: int
}

@export()
func exportedFunc(input string) string => 'Hello ${input}'

// Decorator with argument
@metadata({
  description: 'Parameter with metadata decorator'
  example: 'value'
  required: true
})
param metadataParam string

// Multiple decorators on a single element
@description('Parameter with multiple decorators')
@minLength(5)
@maxLength(50)
@allowed([
  'value1'
  'value2'
  'value3'
])
@secure()
param multipleDecoratorParam string

// Batch size decorator for resources
@batchSize(5)
resource batchResource 'Microsoft.Storage/storageAccounts@2023-04-01' = [
  for i in range(0, 10): {
    name: 'storage${i}'
    location: 'eastus'
    sku: {
      name: 'Standard_LRS'
    }
    kind: 'StorageV2'
  }
]

// Sealed decorator
@sealed()
param sealedObjectParam {
  property1: string
  property2: int
  property3: bool
}

// Decorators on object properties
param objectWithDecoratedProps {
  @description('Name property')
  @minLength(3)
  name: string

  @description('Age property')
  @minValue(0)
  @maxValue(120)
  age: int

  @description('Contact information')
  @metadata({
    required: false
  })
  contact: {
    email: string
    phone: string?
  }
}
