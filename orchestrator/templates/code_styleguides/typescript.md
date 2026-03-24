# TypeScript Style Guide

Based on Google TypeScript Style Guide. Follow these rules for consistent, readable code.

## Variables & Declarations

- Use `const` by default, `let` only when reassignment is needed
- NEVER use `var`
- Use `readonly` for properties that shouldn't change after construction

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Classes, Interfaces, Types, Enums | `UpperCamelCase` | `UserService`, `ApiResponse` |
| Variables, Functions, Methods | `lowerCamelCase` | `getUserById`, `isValid` |
| Constants (global) | `CONSTANT_CASE` | `MAX_RETRY_COUNT` |
| Private members | `lowerCamelCase` (no underscore prefix) | `private userId` |

## Modules & Imports

- Use ES6 imports/exports exclusively
- Use named exports, NOT default exports
- Group imports: external libraries → internal modules → relative imports
- No `namespace` or `module` keywords

```typescript
// Good
import {UserService} from './services/user';
export {UserController};

// Bad
import UserService from './services/user';
export default UserController;
```

## Types

- Prefer `unknown` over `any` - NEVER use `any`
- Use `Array<T>` for complex types, `T[]` for simple types
- Use explicit return types for public APIs
- Use type inference for local variables

```typescript
// Good
function processItems(items: Array<string | number>): void {}
const names: string[] = ['a', 'b'];

// Bad
function processItems(items: any): any {}
```

## Classes

- Use `private` keyword (not `#private`)
- Don't use `public` explicitly (it's default)
- Initialize properties in constructor or declaration

## Equality

- ALWAYS use `===` and `!==`
- NEVER use `==` or `!=`

## Error Handling

- Use typed errors when possible
- Always catch specific errors, not generic `Error`
- Don't ignore caught errors silently

## Documentation

- Use JSDoc for public APIs
- Don't include types in JSDoc (TypeScript infers them)

```typescript
/**
 * Retrieves a user by their unique identifier.
 * @param id - The user's unique identifier
 * @returns The user object or undefined if not found
 */
function getUserById(id: string): User | undefined {}
```

## Formatting

- Use semicolons (don't rely on ASI)
- 2 space indentation
- Single quotes for strings
- Trailing commas in multiline structures
