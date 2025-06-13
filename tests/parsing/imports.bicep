// This file contains examples of import statements in Bicep

// Namespace import
import 'az@1.0.0'

// Module imports with specific symbols
import { myObjectType, sayHello } from './exports.bicep'
import { myObjectType as customObject, sayHello as greeting } from './exports.bicep'

// Module import with wildcard
import * as sampleExports from './exports.bicep'

// Registry module imports
import { roleAssignmentType } from 'br/public:avm/utl/types/avm-common-types:0.5.1'
import * as commonTypes from 'br/avmTypes:avm-common-types:0.5.1'

// TypeSpec module imports
import { deploymentScript } from 'ts/demoSpec:templateSpec:1.0.0'
import * as templates from 'ts:00000000-0000-0000-0000-000000000000/myResourceGroup/myTemplateSpec:v1.0'
