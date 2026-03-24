# JavaScript Style Guide

Based on Google JavaScript Style Guide and Airbnb Style Guide. Follow these rules for consistent, readable code.

## Variables & Declarations

- Use `const` by default
- Use `let` only when reassignment is needed
- NEVER use `var`
- Declare one variable per statement

```javascript
// Good
const name = 'Alice';
let count = 0;

// Bad
var name = 'Alice', count = 0;
```

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Variables, Functions | `camelCase` | `getUserById`, `isValid` |
| Classes | `PascalCase` | `UserService` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_RETRY_COUNT` |
| Private (convention) | `_leadingUnderscore` | `_privateMethod` |

## Modules

- Use ES6 modules (`import`/`export`)
- Prefer named exports over default exports
- Group imports: external → internal → relative

```javascript
// Good
import { useState, useEffect } from 'react';
import { UserService } from './services/user';
export { UserController };

// Avoid
const UserService = require('./services/user');
export default UserController;
```

## Functions

- Use arrow functions for callbacks and anonymous functions
- Use regular functions for methods and constructors
- Prefer function declarations over function expressions

```javascript
// Good
function getUserById(id) {
  return users.find(user => user.id === id);
}

// For callbacks
items.map(item => item.name);
```

## Objects & Arrays

- Use object shorthand
- Use computed property names
- Use spread operator
- Use destructuring

```javascript
// Good
const name = 'Alice';
const user = { name, age: 30 };
const clone = { ...user, city: 'Madrid' };
const { name: userName } = user;

// Avoid
const user = { name: name };
const clone = Object.assign({}, user, { city: 'Madrid' });
```

## Equality

- ALWAYS use `===` and `!==`
- NEVER use `==` or `!=`

## Error Handling

- Always handle Promise rejections
- Use try/catch for async/await
- Don't ignore caught errors

```javascript
// Good
try {
  await fetchData();
} catch (error) {
  console.error('Failed to fetch:', error);
  throw error;
}

// With Promises
fetchData()
  .then(handleData)
  .catch(handleError);
```

## Async/Await

- Prefer async/await over raw Promises
- Use Promise.all for parallel operations
- Always use try/catch

```javascript
// Good
async function fetchUsers() {
  try {
    const [users, roles] = await Promise.all([
      fetchUserList(),
      fetchRoleList()
    ]);
    return { users, roles };
  } catch (error) {
    throw new Error(`Failed to fetch: ${error.message}`);
  }
}
```

## Comments

- Use JSDoc for public APIs
- Use `//` for single-line comments
- Use `/* */` for multi-line comments

```javascript
/**
 * Retrieves a user by their unique identifier.
 * @param {string} id - The user's unique identifier
 * @returns {User|undefined} The user object or undefined
 */
function getUserById(id) {}
```

## Formatting

- Use semicolons
- 2 space indentation
- Single quotes for strings
- Trailing commas in multiline
- Spaces inside curly braces: `{ a, b }`
