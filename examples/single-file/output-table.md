# Example Bicep File

Description of the example Bicep file

Target Scope: resourceGroup

### Additional Metadata

| Key | Value |
|-----|-------|
| author | File Author |

## Imports

*No imports defined*

## Types

### customObject

Type description

| Property | Value |
|----------|-------|
| Type | { name: string, score: int, grade: grade } |
| Exported | No |
| Secure | No |

### grade

| Property | Value |
|----------|-------|
| Type | A \\\| B \\\| C \\\| D \\\| E |
| Exported | Yes |
| Secure | No |

### resourceType

| Property | Value |
|----------|-------|
| Type | { id: string, name: string, resourceGroup: string } |
| Exported | No |
| Secure | No |

## Functions

### generateName

Generate Name Function

| Property | Value |
|----------|-------|
| Return Type | string |
| Exported | Yes |

#### Parameters

| Parameter | Type | Optional |
|-----------|------|----------|
| argument1 | string | No |
| argument2 | int | Yes |

### somethingElse

| Property | Value |
|----------|-------|
| Return Type | bool |
| Exported | No |

## Parameters

### requiredParam

This is a required parameter

| Property | Value |
|----------|-------|
| Type | string |
| Minimum Length | 3 |
| Maximum Length | 10 |

### optionalParam

This is an optional parameter

| Property | Value |
|----------|-------|
| Type | string |
| Nullable | Yes |

### simpleParamWithDefault

| Property | Value |
|----------|-------|
| Type | int |
| Default Value | 100 |
| Minimum Value | 0 |
| Maximum Value | 100 |

### genericObject

I have a description in metadata

| Property | Value |
|----------|-------|
| Type | object |
| Default Value | { name: value, number: 1000 } |

#### Metadata

| Key | Value |
|-----|-------|
| name | A name in metadata |
| somethingElse | Another metadata property |

### inlineSpecificObject

| Property | Value |
|----------|-------|
| Type | { property: string, otionalProperty: int, objectProperty: { key1: string, key2: int } } |

### typedObjects

| Property | Value |
|----------|-------|
| Type | customObject[] |

### individualOptions

| Property | Value |
|----------|-------|
| Type | one \\\| two \\\| three |

### greekLetter

| Property | Value |
|----------|-------|
| Type | string |
| Default Value | alpha |
| Allowed Values | alpha, beta, gamma, delta |

### multiLine

| Property | Value |
|----------|-------|
| Type | string |
| Default Value | \nThis is a multi line string.\n  It covers multiple lines, and has indentation.\n  It also has a tab character ( ).	And a new line.\n  It also has a double backslash \\\\ and a single \\\n  And a single quote: '\n |

## Variables

### nameVar

Variable description

| Property | Value |
|----------|-------|
| Value | someValue |
| Exported | No |

### exportedVar

Exported variable description

| Property | Value |
|----------|-------|
| Value | exportedValue |
| Exported | Yes |

### boolVar

Boolean variable

| Property | Value |
|----------|-------|
| Value | true |
| Exported | Yes |

### numVar

The answer to life, the universe, and everything

| Property | Value |
|----------|-------|
| Value | 42 |
| Exported | No |

## Resources

### storageAccount

| Property | Value |
|----------|-------|
| Type | Microsoft.Storage/storageAccounts |
| API Version | 2023-04-01 |
| Existing | Yes |

### storageAccount::blobServices

| Property | Value |
|----------|-------|
| Type | Microsoft.Storage/storageAccounts/blobServices |
| API Version | 2023-04-01 |
| Existing | Yes |

### storageAccount::blobServices::container

| Property | Value |
|----------|-------|
| Type | Microsoft.Storage/storageAccounts/blobServices/containers |
| API Version | 2023-04-01 |

### vnet

| Property | Value |
|----------|-------|
| Type | Microsoft.Network/virtualNetworks |
| API Version | 2021-05-01 |
| Depends On | roleAssignStorage |

### vnet::defaultSubnet

| Property | Value |
|----------|-------|
| Type | Microsoft.Network/virtualNetworks/subnets |
| API Version | 2021-05-01 |

### vnet::diffApi

| Property | Value |
|----------|-------|
| Type | Microsoft.Network/virtualNetworks/subnets |
| API Version | 2024-05-01 |

### externalChild

Resource Description

| Property | Value |
|----------|-------|
| Type | Microsoft.Network/virtualNetworks/subnets |
| API Version | 2023-11-01 |
| Parent | vnet |
| Condition | (1 == 1) |

### containerLoop

| Property | Value |
|----------|-------|
| Type | Microsoft.Storage/storageAccounts/blobServices/containers |
| API Version | 2024-01-01 |
| Parent | storageAccount::blobServices |
| Loop | for name in ['alice', 'bob', 'charlie'] |
| Batch Size | 2 |

### roleAssignStorage

| Property | Value |
|----------|-------|
| Type | Microsoft.Authorization/roleAssignments |
| API Version | 2022-04-01 |
| Scope | ${storageAccount} |

## Modules

*No modules defined*

## Outputs

### one

Output Description

| Property | Value |
|----------|-------|
| Type | string |
| Value | one |
| Secure | Yes |

### storageAccountOutput

| Property | Value |
|----------|-------|
| Type | resourceType |
| Value | { id: storageAccount.id, name: storageAccount.name, resourceGroup: resourceGroup().name } |

### percentage

| Property | Value |
|----------|-------|
| Type | int |
| Value | true ? 50 : 100 |
| Minimum Value | 0 |
| Maximum Value | 100 |

### fib

| Property | Value |
|----------|-------|
| Type | string[] |
| Value | [1, 1, 2, 3, 5, 8] |
| Minimum Length | 1 |
| Maximum Length | 34 |

