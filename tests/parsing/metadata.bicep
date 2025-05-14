// Metadata Test File
// Tests various metadata declarations and features

// Simple metadata declarations
metadata name = 'Metadata Test File'
metadata description = 'This file tests various metadata declarations in Bicep'
metadata version = '1.0.0'

// Metadata with string value
metadata author = 'Test Author'

// Metadata with integer value
metadata count = 42

// Metadata with boolean value
metadata enabled = true

// Metadata with object value
metadata object = {
  prop1: 'value1'
  prop2: 'value2'
  nested: {
    nestedProp: 'nested value'
  }
}

// Metadata with array value
metadata tags = [
  'tag1'
  'tag2'
  'tag3'
]

// Metadata for configuration
metadata configuration = {
  debug: true
  environment: 'test'
  timeout: 30
}

// Complex metadata example
metadata deployment = {
  creator: 'automated-process'
  settings: {
    retryCount: 3
    retryInterval: 10
    notifications: [
      {
        type: 'email'
        recipient: 'admin@example.com'
      }
      {
        type: 'webhook'
        url: 'https://example.com/webhook'
      }
    ]
  }
}
