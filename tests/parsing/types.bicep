// Complex Type Example

@description('Environment name for the deployment')
param environmentName string = 'dev'

@description('The location where resources will be deployed')
param location string = resourceGroup().location

@description('Tags to apply to all resources')
param tags object = {
  Environment: environmentName
  Project: 'Bicep Test'
  CreatedBy: 'Document Parser'
}

@description('Configuration for the storage accounts')
param storageConfig object = {
  storageAccountPrefix: 'st'
  skuName: 'Standard_LRS'
  kind: 'StorageV2'
  accessTier: 'Hot'
  supportsHttpsTrafficOnly: true
  minimumTlsVersion: 'TLS1_2'
  allowBlobPublicAccess: false
  networkRules: {
    defaultAction: 'Deny'
    bypass: ['AzureServices']
    virtualNetworkRules: []
    ipRules: []
  }
}

@description('Application settings as key-value pairs')
param appSettings array = [
  {
    name: 'WEBSITE_NODE_DEFAULT_VERSION'
    value: '~16'
  }
  {
    name: 'APPINSIGHTS_INSTRUMENTATIONKEY'
    value: 'placeholder'
  }
  {
    name: 'ENVIRONMENT'
    value: environmentName
  }
]

@description('Complex type for virtual network configuration')
type NetworkConfig = {
  @description('Virtual network name')
  vnetName: string

  @description('Virtual network address prefix')
  addressPrefix: string

  @description('Subnet configurations')
  subnets: Subnet[]

  @description('Enable DDoS protection')
  enableDdosProtection: bool?
}

@description('Subnet definition')
type Subnet = {
  @description('Subnet name')
  name: string

  @description('Subnet address prefix')
  addressPrefix: string

  @description('Network security group to associate')
  nsgId: string?

  @description('Route table to associate')
  routeTableId: string?
}

@description('This parameter uses the custom NetworkConfig type')
param networkParameters NetworkConfig = {
  vnetName: 'vnet-${environmentName}'
  addressPrefix: '10.0.0.0/16'
  subnets: [
    {
      name: 'default'
      addressPrefix: '10.0.0.0/24'
      nsgId: null
      routeTableId: null
    }
    {
      name: 'app'
      addressPrefix: '10.0.1.0/24'
      nsgId: null
      routeTableId: null
    }
  ]
  enableDdosProtection: false
}

// Storage account resource with configuration from parameter
resource storageAccount 'Microsoft.Storage/storageAccounts@2021-06-01' = {
  name: '${storageConfig.storageAccountPrefix}${uniqueString(resourceGroup().id)}'
  location: location
  tags: tags
  sku: {
    name: storageConfig.skuName
  }
  kind: storageConfig.kind
  properties: {
    accessTier: storageConfig.accessTier
    supportsHttpsTrafficOnly: storageConfig.supportsHttpsTrafficOnly
    minimumTlsVersion: storageConfig.minimumTlsVersion
    allowBlobPublicAccess: storageConfig.allowBlobPublicAccess
    networkAcls: storageConfig.networkRules
  }
}

// App Service Plan resource
resource appServicePlan 'Microsoft.Web/serverfarms@2021-02-01' = {
  name: 'app-service-plan-${environmentName}'
  location: location
  tags: tags
  sku: {
    name: 'B1'
    tier: 'Basic'
  }
  properties: {
    reserved: true
  }
}

// Function App resource
resource functionApp 'Microsoft.Web/sites@2021-02-01' = {
  name: 'function-app-${environmentName}'
  location: location
  tags: tags
  kind: 'functionapp,linux'
  properties: {
    serverFarmId: appServicePlan.id
    siteConfig: {
      appSettings: appSettings
      linuxFxVersion: 'Node|16'
    }
  }
}

// Virtual network resource based on networkParameters
resource virtualNetwork 'Microsoft.Network/virtualNetworks@2021-02-01' = {
  name: networkParameters.vnetName
  location: location
  tags: tags
  properties: {
    addressSpace: {
      addressPrefixes: [
        networkParameters.addressPrefix
      ]
    }
    subnets: [
      for subnet in networkParameters.subnets: {
        name: subnet.name
        properties: {
          addressPrefix: subnet.addressPrefix
          networkSecurityGroup: subnet.nsgId == null
            ? null
            : {
                id: subnet.nsgId
              }
          routeTable: subnet.routeTableId == null
            ? null
            : {
                id: subnet.routeTableId
              }
        }
      }
    ]
    enableDdosProtection: networkParameters.enableDdosProtection ?? false
  }
}

output storageAccountName string = storageAccount.name
output storageAccountId string = storageAccount.id
output functionAppName string = functionApp.name
output virtualNetworkName string = virtualNetwork.name
output subnetNames array = [for subnet in networkParameters.subnets: subnet.name]
