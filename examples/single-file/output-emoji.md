# Example Bicep File

Description of the example Bicep file

**Target Scope:** `resourceGroup`

## Additional Metadata

**author:** File Author

## Imports

*No imports defined*

## Types

### `customObject`

Type description

**Type:** `object`  
**Exported:** ❌ No  
**Nullable:** ❌ No  
**Secure:** ❌ No  

**Object Definition**

#### `name`

**Type:** `string`  
**Nullable:** ✅ Yes  
**Secure:** ✅ Yes  

#### `score`

**Type:** `int`  
**Nullable:** ❌ No  
**Secure:** ❌ No  

**Constraints**

**Minimum Value:** `0`  
**Maximum Value:** `100`  

#### `grade`

**Type:** `grade`  
**Nullable:** ❌ No  
**Secure:** ❌ No  


### `grade`

**Type:** `'A' | 'B' | 'C' | 'D' | 'E'`  
**Exported:** ✅ Yes  
**Nullable:** ❌ No  
**Secure:** ❌ No  

### `resourceType`

**Type:** `object`  
**Exported:** ❌ No  
**Nullable:** ❌ No  
**Secure:** ❌ No  

**Object Definition**

#### `id`

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ❌ No  

#### `name`

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ❌ No  

#### `resourceGroup`

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ❌ No  


## Functions

### `generateName`

Generate Name Function

**Return Type:** `string`  
**Exported:** ✅ Yes  

**Parameters**

**argument1:** `string`
**argument2:** `int` (Optional)

**Definition**

```bicep
toLower('${argument1}-${argument2}')
```

**Metadata**

**description:** Generate Name Function


### `somethingElse`

**Return Type:** `bool`  
**Exported:** ❌ No  

**Definition**

```bicep
true
```

## Parameters

### `requiredParam`

This is a required parameter

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ✅ Yes  
**Sealed:** ❌ No  

**Constraints**

**Minimum Length:** `3`  
**Maximum Length:** `10`  

### `optionalParam`

This is an optional parameter

**Type:** `string`  
**Nullable:** ✅ Yes  
**Secure:** ❌ No  
**Sealed:** ❌ No  

### `simpleParamWithDefault`

**Type:** `int`  
**Nullable:** ❌ No  
**Secure:** ❌ No  
**Sealed:** ❌ No  

**Constraints**

**Minimum Value:** `0`  
**Maximum Value:** `100`  

**Default Value**

```bicep
100
```

### `genericObject`

I have a description in metadata

**Metadata**

**name:** A name in metadata

**somethingElse:** Another metadata property


**Type:** `object`  
**Nullable:** ❌ No  
**Secure:** ❌ No  
**Sealed:** ❌ No  

**Default Value**

```bicep
{ name: value, number: 1000 }
```

### `inlineSpecificObject`

**Type:** `object`  
**Nullable:** ❌ No  
**Secure:** ❌ No  
**Sealed:** ✅ Yes  

**Object Definition**

#### `property`

Description of the property

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ✅ Yes  

**Constraints**

**Maximum Length:** `10`  

#### `otionalProperty`

**Type:** `int`  
**Nullable:** ✅ Yes  
**Secure:** ❌ No  

**Constraints**

**Minimum Value:** `23`  

#### `objectProperty`

**Type:** `object`  
**Nullable:** ❌ No  
**Secure:** ❌ No  

**Object Definition**

##### `key1`

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ❌ No  

##### `key2`

**Type:** `int`  
**Nullable:** ❌ No  
**Secure:** ❌ No  



### `typedObjects`

**Type:** `customObject[]`  
**Nullable:** ❌ No  
**Secure:** ❌ No  
**Sealed:** ❌ No  

### `individualOptions`

**Type:** `'one' | 'two' | 'three'`  
**Nullable:** ❌ No  
**Secure:** ❌ No  
**Sealed:** ❌ No  

### `greekLetter`

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ❌ No  
**Sealed:** ❌ No  

**Constraints**

**Allowed Values:**   
`alpha`    
`beta`    
`gamma`    
`delta`  

**Default Value**

```bicep
alpha
```

### `multiLine`

**Type:** `string`  
**Nullable:** ❌ No  
**Secure:** ❌ No  
**Sealed:** ❌ No  

**Default Value**

```bicep

This is a multi line string.
  It covers multiple lines, and has indentation.
  It also has a tab character (	).	And a new line.
  It also has a double backslash \\ and a single \
  And a single quote: '

```

## Variables

### `nameVar`

Variable description

**Exported:** ❌ No  

**Value**

```bicep
someValue
```

### `exportedVar`

Exported variable description

**Exported:** ✅ Yes  

**Value**

```bicep
exportedValue
```

### `boolVar`

Boolean variable

**Exported:** ✅ Yes  

**Value**

```bicep
true
```

### `numVar`

The answer to life, the universe, and everything

**Exported:** ❌ No  

**Value**

```bicep
42
```

## Resources

### `storageAccount`

**Name:** `mystorageaccount`  
**Type:** `Microsoft.Storage/storageAccounts`  
**API Version:** `2023-04-01`  
**Existing:** ✅ Yes  

### `storageAccount::blobServices`

**Name:** `default`  
**Type:** `Microsoft.Storage/storageAccounts/blobServices`  
**API Version:** `2023-04-01`  
**Existing:** ✅ Yes  

### `storageAccount::blobServices::container`

**Name:** `myContainer`  
**Type:** `Microsoft.Storage/storageAccounts/blobServices/containers`  
**API Version:** `2023-04-01`  

### `vnet`

**Name:** `${nameVar}`  
**Type:** `Microsoft.Network/virtualNetworks`  
**API Version:** `2021-05-01`  
**Depends On:** `roleAssignStorage`  

### `vnet::defaultSubnet`

**Name:** `default`  
**Type:** `Microsoft.Network/virtualNetworks/subnets`  
**API Version:** `2021-05-01`  

### `vnet::diffApi`

**Name:** `api`  
**Type:** `Microsoft.Network/virtualNetworks/subnets`  
**API Version:** `2024-05-01`  

### `externalChild`

Resource Description

**Name:** `another`  
**Type:** `Microsoft.Network/virtualNetworks/subnets`  
**API Version:** `2023-11-01`  
**Parent:** `vnet`  
**Condition:**   
  
```bicep  
(1 == 1)  
```  
  

### `containerLoop`

**Name:** `container${name}`  
**Type:** `Microsoft.Storage/storageAccounts/blobServices/containers`  
**API Version:** `2024-01-01`  
**Parent:** `storageAccount::blobServices`  
**Batch Size:** `2`  
**Loop:**   
```bicep  
for name in ['alice', 'bob', 'charlie']  
```  
  

### `roleAssignStorage`

**Name:** `guid(storageAccount.name)`  
**Type:** `Microsoft.Authorization/roleAssignments`  
**API Version:** `2022-04-01`  
**Scope:** `${storageAccount}`  

## Modules

*No modules defined*

## Outputs

### `one`

Output Description

**Type:** `string`  
**Sealed:** ❌ No  
**Secure:** ✅ Yes  

**Value**

```bicep
one
```

### `storageAccountOutput`

**Type:** `resourceType`  
**Sealed:** ❌ No  
**Secure:** ❌ No  

**Value**

```bicep
{ id: storageAccount.id, name: storageAccount.name, resourceGroup: resourceGroup().name }
```

### `percentage`

**Type:** `int`  
**Sealed:** ❌ No  
**Secure:** ❌ No  

**Constraints**

**Minimum Value:** `0`  
**Maximum Value:** `100`  

**Value**

```bicep
true ? 50 : 100
```

### `fib`

**Type:** `string[]`  
**Sealed:** ❌ No  
**Secure:** ❌ No  

**Constraints**

**Minimum Length:** `1`  
**Maximum Length:** `34`  

**Value**

```bicep
[1, 1, 2, 3, 5, 8]
```

