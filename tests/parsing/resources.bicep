// Resources Test File
// Tests various resource declarations and features

// Simple resource declaration
resource storageAccount 'Microsoft.Storage/storageAccounts@2023-04-01' = {
  name: 'myteststorageaccount'
  location: 'eastus'
  sku: {
    name: 'Standard_LRS'
  }
  kind: 'StorageV2'

  resource fileServices 'fileServices' = {
    name: 'default'
  }
}

// Resource with description
@description('Virtual network resource')
resource vnet 'Microsoft.Network/virtualNetworks@2021-05-01' = {
  name: 'mytestvnet'
  location: 'eastus'
  properties: {
    addressSpace: {
      addressPrefixes: [
        '10.0.0.0/16'
      ]
    }
  }
}

// Existing resource reference
resource existingStorage 'Microsoft.Storage/storageAccounts@2023-04-01' existing = {
  name: 'existingstorage'
}

// Child resource using parent property
resource subnet 'Microsoft.Network/virtualNetworks/subnets@2021-05-01' = {
  parent: vnet
  name: 'default'
  properties: {
    addressPrefix: '10.0.0.0/24'
  }
}

// Nested child resource
resource blobServices 'Microsoft.Storage/storageAccounts/blobServices@2023-04-01' = {
  parent: storageAccount
  name: 'default'

  resource container 'containers' = {
    name: 'mycontainer'
  }
}

// Resource with different API version
resource diffApiSubnet 'Microsoft.Network/virtualNetworks/subnets@2022-07-01' = {
  parent: vnet
  name: 'api'
  properties: {
    addressPrefix: '10.1.0.0/24'
  }
}

// Conditional resource
@description('Conditional resource')
resource conditionalResource 'Microsoft.Network/networkSecurityGroups@2021-05-01' = if (true) {
  name: 'mynsg'
  location: 'eastus'
}

// Resource with scope
resource roleAssignment 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  scope: storageAccount
  name: 'myroleassignment'
  properties: {
    roleDefinitionId: '00000000-0000-0000-0000-000000000000'
    principalId: '00000000-0000-0000-0000-000000000000'
  }
}

// Resource with dependsOn
resource dependentResource 'Microsoft.Network/publicIPAddresses@2021-05-01' = {
  name: 'mypublicip'
  location: 'eastus'
  properties: {
    publicIPAllocationMethod: 'Dynamic'
  }
  dependsOn: [
    vnet
  ]
}

// Resource with loop
@batchSize(2)
resource resourceLoop 'Microsoft.Storage/storageAccounts/blobServices/containers@2023-04-01' = [
  for name in ['container1', 'container2', 'container3']: {
    parent: blobServices
    name: name
  }
]

// Resource using double-colon syntax for referencing child
resource childReference 'Microsoft.Storage/storageAccounts/fileServices/shares@2023-04-01' = {
  parent: storageAccount::fileServices
  name: 'childshare'
}

// Variables for testing identifier references
var nameVar = 'identifierNameTest'
var locationVar = 'westus'
var dependsOnVar = [vnet, storageAccount]

// Resource with identifier references in properties
resource identifierResource 'Microsoft.Storage/storageAccounts@2023-04-01' = {
  name: nameVar
  location: locationVar
  sku: {
    name: 'Standard_LRS'
  }
  kind: 'StorageV2'
  dependsOn: dependsOnVar
}
