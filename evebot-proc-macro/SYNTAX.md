# Macro Configuration Syntax

## Service

```json5
{
  "title": "string | Some Title",
  "description": "string | Some description.",
  "arg_prefix": "string | perf1 perf2 perf3",
  "positional_args": [
    {
      "arg_name": "string | argument_name",
      "arg_type": "<arg-type>",
      "description": "string | Some description."
    }
  ],
  "param_args": [
    {
      "arg_name": "string | parameter_name",
      "description": "string | Some description.",
      "arg_type": "<arg-type>",
      "default": "<arg-value>",   // optional
      "alias": [                  // optional
        "string | alias_name"
      ]
    }
  ],
  "mixin": [    // optional
    "path/to/mixin.json"
  ]
}
```

### Arg-Type

```json5
{
  "AnyText": "AnyText",
  "EnumText": {
    "EnumText": [
      "string | some-text"
    ]
  },
  "Float": "Float",
  "Int": "Int",
  "Boolean": "Boolean"
}
```

### Arg-Value

```json5
{
  "AnyText": {
    "AnyText": "string | some-text"
  },
  "EnumText": {
    "EnumText": "string | some-text"
  },
  "Float": {
    "Float": 114.514      // float
  },
  "Int": {
    "Int": 114514         // integer
  },
  "Boolean": {
    "Boolean": true       // boolean
  }
}
```

## Distributor

### Meta Structure

#### SubGroups

```json5
{
  "path-ident": "string | ident",
  "path-alias": [     // optional
    "string | ident-alias"
  ],
  "description": "string | Some description.",
  "group-name": "string | Group Name",
  "subcommand": [     // optional
    "meta-structure | subcommand"
  ],
  "subgroup": [       // optional
    "meta-structure | subgroup"
  ]
}
```

#### SubCommands

```json5
{
  "path-ident": "string | ident",
  "path-alias": [     // optional
    "string | ident-alias"
  ],
  "structure-path": "string | rust-like path | crate::foo::bar::FooBar",
  "description": "string | Some description.",
  "no-help": false  // No help for this command. Default: false.
}
```
