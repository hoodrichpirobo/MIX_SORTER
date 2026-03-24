# Go Style Guide

Based on Effective Go and Google Go Style Guide. Follow these rules for idiomatic Go code.

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Packages | `lowercase`, short, no underscores | `user`, `httputil` |
| Exported (public) | `PascalCase` | `UserService`, `GetByID` |
| Unexported (private) | `camelCase` | `userService`, `getByID` |
| Interfaces (single method) | Method name + `er` | `Reader`, `Writer`, `Stringer` |
| Acronyms | All caps or all lower | `HTTPServer`, `xmlParser` |

## Package Organization

- One package per directory
- Package name = directory name
- Keep packages focused and small
- Avoid `util`, `common`, `misc` packages

```go
// Good
package user

// Bad
package userService
package user_service
```

## Imports

- Use `goimports` to organize
- Group: standard library → external → internal
- Avoid dot imports
- Use blank identifier for side effects only

```go
import (
    "context"
    "fmt"

    "github.com/gin-gonic/gin"

    "myapp/internal/user"
)
```

## Error Handling

- Return errors, don't panic
- Handle errors immediately
- Wrap errors with context
- Use `errors.Is` and `errors.As` for comparison

```go
// Good
user, err := GetUser(id)
if err != nil {
    return nil, fmt.Errorf("failed to get user %s: %w", id, err)
}

// Bad
user, _ := GetUser(id)  // ignoring error
```

## Functions

- Return early on errors
- Keep functions small and focused
- Accept interfaces, return structs
- Use named return values sparingly

```go
// Good
func GetUser(id string) (*User, error) {
    if id == "" {
        return nil, errors.New("id cannot be empty")
    }
    // ... rest of function
}
```

## Structs

- Use struct literals with field names
- Group related fields
- Use embedding for composition

```go
// Good
user := User{
    Name:  "Alice",
    Email: "alice@example.com",
}

// Bad
user := User{"Alice", "alice@example.com"}
```

## Interfaces

- Keep interfaces small (1-3 methods)
- Define interfaces where they're used, not where implemented
- Use standard interfaces when possible (`io.Reader`, `fmt.Stringer`)

```go
// Good - small, focused interface
type UserRepository interface {
    GetByID(ctx context.Context, id string) (*User, error)
}

// Bad - too many methods
type UserRepository interface {
    GetByID(id string) (*User, error)
    GetByEmail(email string) (*User, error)
    Create(user *User) error
    Update(user *User) error
    Delete(id string) error
    List() ([]*User, error)
}
```

## Context

- Pass context as first parameter
- Don't store context in structs
- Use context for cancellation and deadlines

```go
func GetUser(ctx context.Context, id string) (*User, error) {
    // ...
}
```

## Concurrency

- Use goroutines for concurrent work
- Use channels for communication
- Use sync package for synchronization
- Prefer `sync.WaitGroup` for waiting on goroutines

```go
func ProcessItems(items []Item) {
    var wg sync.WaitGroup
    for _, item := range items {
        wg.Add(1)
        go func(i Item) {
            defer wg.Done()
            process(i)
        }(item)
    }
    wg.Wait()
}
```

## Comments

- Write godoc comments for exported identifiers
- Start with the name of the thing being described
- Full sentences with periods

```go
// User represents a registered user in the system.
type User struct {
    ID   string
    Name string
}

// GetByID retrieves a user by their unique identifier.
// It returns nil if no user is found.
func GetByID(id string) *User {
    // ...
}
```

## Formatting

- Use `gofmt` or `goimports` (mandatory)
- Tabs for indentation
- No line length limit, but be reasonable
