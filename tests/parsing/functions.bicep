// Simple function without parameters
func simpleFunction() bool => true

// Function with one string parameter
func oneParamFunction(name string) string => name

// Function with a nullable parameter
func nullableParamFunction(id int?) int => 42

// Function with multiple parameters of different types
func multiParamFunction(name string, age int, active bool) object => {
  displayName: name
  userAge: age
  isActive: active
}

// Function with a complex return type
func arrayFunction() string[] => [
  'one'
  'two'
  'three'
]

// Function with decorators
@description('This function calculates the full name')
@export()
func getFullName(firstName string, lastName string) string => '${firstName} ${lastName}'

// Function with export decorator only
@export()
func exportedFunction(value string) bool => true

// Function with a custom type as parameter
func customTypeFunction(user customUser) string => user.name

// Function returning a custom type
func returnCustomType() customUser => {
  name: 'John'
  email: 'john@example.com'
}

// Custom type for testing
type customUser = {
  name: string
  email: string
}

// Function with complex parameter types
func complexParamFunction(users customUser[], filter object) customUser[] => []
