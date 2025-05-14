@description('Module Example File')
metadata name = 'Module Examples'
metadata description = 'Examples of different module types in Bicep'

// Local module example
@description('Local module example')
module localModule './path/to/module.bicep' = {
  name: 'localModuleDeployment'
  params: {
    location: 'eastus'
    environmentName: 'prod'
  }
}

// Registry module example with alias
@description('Registry module with alias')
module registryWithAlias 'br/myRegistry:module/path:v1.0' = {
  name: 'registryModuleWithAliasDeployment'
  params: {
    sku: 'Standard'
    instances: 3
  }
}

// Registry module example with FQDN
@description('Registry module with FQDN')
module registryWithFQDN 'br:mcr.microsoft.com/architecture/webapp:1.2.3' = {
  name: 'registryModuleWithFQDNDeployment'
  params: {
    location: 'westus'
  }
}

// TypeSpec module with alias
@description('TypeSpec module with alias')
module typeSpecWithAlias 'ts/myAlias:myTemplateSpec:v2.0' = {
  name: 'typeSpecModuleWithAliasDeployment'
  params: {
    environment: 'staging'
  }
}

// TypeSpec module with subscription/resource group
@description('TypeSpec module with subscription and resource group')
module typeSpecWithSub 'ts:00000000-0000-0000-0000-000000000000/myResourceGroup/myTemplateSpec:v1.0' = {
  name: 'typeSpecModuleWithSubDeployment'
  params: {
    tier: 'premium'
  }

  dependsOn: [
    typeSpecWithAlias
  ]
}

// Conditional module
@description('Conditional module')
module conditionalModule 'modules/conditional.bicep' = if (true) {
  name: 'conditionalModuleDeployment'
  params: {
    enabled: true
  }
}

// Module with dependencies
@description('Module with dependencies')
module dependencyModule 'modules/dependencies.bicep' = {
  name: 'dependencyModuleDeployment'
  params: {
    resourceName: 'myResource'
  }
  dependsOn: [
    localModule
    conditionalModule
  ]
}

// Module loop
@description('Module loop')
@batchSize(2)
module loopModule 'modules/loop.bicep' = [
  for item in ['item1', 'item2', 'item3']: {
    name: 'loopModule-${item}'
    params: {
      itemName: item
    }
  }
]
