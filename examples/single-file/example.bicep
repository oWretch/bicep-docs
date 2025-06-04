metadata name = 'Example Bicep File'
metadata description = 'Description of the example Bicep file'
metadata author = 'File Author'

targetScope = 'resourceGroup'

@description('This is a required parameter')
@minLength(3)
@maxLength(10)
@secure()
param requiredParam string

@sys.description('This is an optional parameter')
@export()
param optionalParam string?

// I don't have a description
@minValue(0)
@maxValue(100)
param simpleParamWithDefault int = 100

@metadata({
  description: 'I have a description in metadata'
  name: 'A name in metadata'
  somethingElse: 'Another metadata property'
})
param genericObject object = {
  name: 'value'
  number: 1000
}

@sealed()
param inlineSpecificObject {
  @maxLength(10)
  @secure()
  @description('Description of the property')
  property: string
  @minValue(23)
  otionalProperty: int?
  objectProperty: {
    key1: string
    key2: int
  }
}

param typedObjects customObject[]

param individualOptions 'one' | 'two' | 'three'

@allowed(['alpha', 'beta', 'gamma', 'delta'])
param greekLetter string = 'alpha'

param multiLine string = '''
This is a multi line string.
  It covers multiple lines, and has indentation.
  It also has a tab character (	).	And a new line.
  It also has a double backslash \\ and a single \
  And a single quote: '
'''

@sys.description('Type description')
type customObject = {
  @secure()
  name: string?
  @minValue(0)
  @maxValue(100)
  score: int
  grade: grade
}

@export()
type grade = 'A' | 'B' | 'C' | 'D' | 'E'

type resourceType = {
  id: string
  name: string
  resourceGroup: string
}

@description('Generate Name Function')
@export()
func generateName(argument1 string, argument2 int?) string =>
  toLower('${argument1}-${argument2}')

func somethingElse() bool => true

@description('Variable description')
var nameVar = 'someValue'

@sys.description('Exported variable description')
@export()
var exportedVar = 'exportedValue'

@sys.description('Boolean variable')
@export()
var boolVar = true

@sys.description('The answer to life, the universe, and everything')
var numVar = 42

resource storageAccount 'Microsoft.Storage/storageAccounts@2023-04-01' existing = {
  name: 'mystorageaccount'

  resource blobServices 'blobServices' existing = {
    name: 'default'

    resource container 'containers' = {
      name: 'myContainer'
    }
  }
}

resource vnet 'Microsoft.Network/virtualNetworks@2021-05-01' = {
  name: nameVar
  location: 'australiaeast'
  properties: {
    addressSpace: {
      addressPrefixes: [
        '10.0.0.0/16'
      ]
    }
  }

  resource defaultSubnet 'subnets' = {
    name: 'default'
    properties: {
      addressPrefix: '10.0.0.0/24'
    }
  }
  resource diffApi 'subnets@2024-05-01' = {
    name: 'api'
    properties: {
      addressPrefix: '10.2.0.0/24'
    }
  }

  dependsOn: [roleAssignStorage]
}

@description('Resource Description')
resource externalChild 'Microsoft.Network/virtualNetworks/subnets@2023-11-01' = if (1 == 1) {
  parent: vnet
  name: 'another'
  properties: {
    addressPrefix: '10.1.0.0/24'
  }
}

@batchSize(2)
resource containerLoop 'Microsoft.Storage/storageAccounts/blobServices/containers@2024-01-01' = [
  for name in ['alice', 'bob', 'charlie']: {
    parent: storageAccount::blobServices
    name: 'container${name}'
  }
]

resource roleAssignStorage 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  scope: storageAccount
  name: guid(storageAccount.name)
  properties: {
    roleDefinitionId: '00000000-0000-0000-0000-000000000000'
    principalId: '00000000-0000-0000-0000-000000000000'
  }
}

@secure()
@sys.description('Output Description')
output one string = 'one'

output storageAccountOutput resourceType = {
  id: storageAccount.id
  name: storageAccount.name
  resourceGroup: resourceGroup().name
}

@minValue(0)
@maxValue(100)
output percentage int = true ? 50 : 100

@maxLength(34)
@minLength(1)
output fib array = [1, 1, 2, 3, 5, 8]
