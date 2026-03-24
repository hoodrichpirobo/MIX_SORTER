# Python Style Guide

Based on Google Python Style Guide and PEP 8. Follow these rules for consistent, readable code.

## Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Modules, Functions, Variables | `snake_case` | `user_service.py`, `get_user_by_id` |
| Classes | `PascalCase` | `UserService`, `ApiResponse` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_RETRY_COUNT` |
| Private/Internal | `_leading_underscore` | `_internal_helper` |

## Imports

- One import per line
- Group imports: standard library → third-party → local
- Use absolute imports over relative
- Avoid `from module import *`

```python
# Good
import os
import sys

from flask import Flask, request
from requests import Session

from myapp.services import user_service
from myapp.models import User

# Bad
from os import *
import os, sys
```

## Line Length & Formatting

- Maximum 80 characters per line
- 4 spaces indentation (NEVER tabs)
- 2 blank lines between top-level definitions
- 1 blank line between methods

## Type Annotations

- Use type hints for public APIs
- Use `Optional[T]` for nullable types
- Use `typing` module for complex types

```python
from typing import Optional, List, Dict

def get_user_by_id(user_id: str) -> Optional[User]:
    """Retrieves a user by their unique identifier."""
    pass

def process_items(items: List[str]) -> Dict[str, int]:
    pass
```

## Docstrings

- Use triple double quotes `"""`
- One-line summary for simple functions
- Full docstring with Args, Returns, Raises for complex functions

```python
def get_user_by_id(user_id: str) -> Optional[User]:
    """Retrieves a user by their unique identifier.

    Args:
        user_id: The user's unique identifier.

    Returns:
        The User object if found, None otherwise.

    Raises:
        ValueError: If user_id is empty.
    """
    pass
```

## Error Handling

- NEVER use bare `except:`
- Catch specific exceptions
- Use `raise` without argument to re-raise

```python
# Good
try:
    process()
except ValueError as e:
    logger.error(f"Invalid value: {e}")
    raise

# Bad
try:
    process()
except:
    pass
```

## Classes

- Use `@property` for computed attributes
- Use `@staticmethod` or `@classmethod` appropriately
- Prefer composition over inheritance

## Strings

- Use f-strings for formatting (Python 3.6+)
- Be consistent with quote style (single or double)
- Use triple quotes for multi-line strings

```python
# Good
name = "Alice"
message = f"Hello, {name}!"

# Avoid
message = "Hello, " + name + "!"
message = "Hello, {}!".format(name)
```

## Boolean Expressions

- Use `is` for None comparisons: `if x is None:`
- Use implicit boolean evaluation: `if items:` not `if len(items) > 0:`
- Don't compare to True/False: `if valid:` not `if valid == True:`

## Linting

- Run `pylint` or `flake8` before committing
- Run `black` or `autopep8` for formatting
- Run `mypy` for type checking
