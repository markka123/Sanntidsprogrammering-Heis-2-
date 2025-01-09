NOT
----------------------


Package
- Programs start running in package main
- Place imports into a parethesized import statement
- Exported names from packages are to start with a capital letter (math.Pi)
- The type-declaration (e.g int) comes after the variable name
- When consecutive function params are the same type, the type only needs to be declared for the last variable (func two_ints(x, y int) (int, int) {return x, y})
- A function can return any number of results, and their types need to be stated within a parenthesis in the function declaration
- Do not need semicolons etc
- Can have naked return statements but this can harm readability
- The "var" statement declares a list of variables and can include initialiers (var boolean1, boolean2, boolean3 bool = True, false, false)
- Inside a function ":=" can be used inplace of the var declaration with implicit type (k := 3)
- Outside a function every statemens begins with a keyword so this shortcut is not applicable
- Variables declared without an explicit initial value are given their zero-value (numeric: 0, string: "", boolean: false)
- in go assignment between different types requieres an explicit conversion
- Constants are declared with the const keyword