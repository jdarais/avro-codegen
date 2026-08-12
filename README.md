# avro-codegen
Avro-codegen is a universal code generator for avro schemas.  It allows you to generate data structures in various programming languages that can be used for convenient serialization / deserialization and that can be checked by the compiler or type checker at build-time.  Avro-codegen comes with built-in generators, and also makes it easy to create a custom generator if the available built-in generators don't meet your needs.  Generators are written in a combination of [Lua](https://lua.org) script and [Tera](https://keats.github.io/tera/docs/) templates.

This is alpha/experimental software

## Using the avro_codegen CLI

The code generator can be run by passing your project directory to the avro_codegen CLI. For example, to run code generation for the `sample_schemas` example project in this repository, run:

```sh
avro_codegen generate examples/sample_schemas
```

Output of `avro_codegen --help`:

```
Usage: avro_codegen <COMMAND>

Commands:
  generate  Run code generation
  show      Display information about a code generator
  list      List available generators
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Built-in Generators

The following generators come built-in with avro-codegen:

- Rust
- TypeScript
- C++

## The avro_codegen Project Directory

An avro_codegen project directory contains an `avro_codegen.toml` file and any number of avro schema (.avsc) files.  For examples, see the `examples/` directory in this repository.

The `avro_codegen.toml` file supports the following properties

- **name**: (required): THe name of the project
- **version**: (required): The project version
- **description**: The project description
- **include**: A list of glob patterns to search for schema files to include, (usually something like ["src/**/*.avsc"])
- **default_generators**: A list of generators to invoke by default when no specific generator has been selected for the "generate" command
- **generators**: A dictionary mapping generator names to custom generators.  Generators have the following properties:
  - **path**: Path (relative to project root) to the generator

## Creating a generator

A generator contains:

- A `generator.toml` file that provides metadata (name, description, etc.) about the generator
- A `generator.lua` script that invokes templates to render code (or other) files
- A `templates/` direcotry containing a collection of tera templates that can be invoked by the generator script

Examples can be found in `src/generators`

### Variables Passed to Generators

Avro_codegen sets the following global variables when invoking the generator's `generate.lua` script:

- `schemas` - A list of all named schemas (records, enums, and fixeds,) defined in the project.  Each list item is a structure describing the schema.
- `package` - Metadata about the project, (e.g. name, version, description, etc.)
- `params` - Parameter values set either in the project config file or on the command line

#### Schemas

All schemas in the `schemas` list, (and any schemas they contain,) have the fields:

- `type` - A string representing schema type.  This can be any avro type, or `ref` if it's a named type that is contained within another schema.
- `json` - The full, self-contained avro schema json

All named types contain:

- `name` - The name of the schema
- `namespace` - The namespace of the schema
- `fullname` - The fully-qualified name of the schema (i.e. `<namespace>.<name>`)

All records contain:

- `fields` - An array of field objects.  Each field object contains a `name` field containing the field name and a `type` field containing the field's schema

All enum records contain:

- `symbols` - An array of strings representing the possible enum values
- `default` - A string representing the default enum value

All union records contain:

- `variants` - An array of schemas representing the variants of the union, in order

All fixed records contain:

- `size` - An integer representing the fixed type's size

All array schemas contain:

- `items` - A schema object representing the type of the array itmes.  This will always be an object, even if the type in the original schema was just a string representing a primitive name.

All map schemas contain:

- `values` - A schema object representing the type of the map values.  This will always be an object, even if the type in the original schema was just a string representing a primitive name.

All schemas with a logical type contain:

- `logical_type` - A string representing the logical type of the schema

All schemas with a decimal logical type contain:

- `precision` - An integer representing the decimal precision
- `scale` - An integer representing the decimal scale

All schemas with a ref type contain:

- `ref_type` - The concrete type of the referenced type, (i.e. "record", "enum" or "fixed").

#### Package

Package properties include:

- `name` - The package name
- `version` - The package version
- `description` - The package description

### Examples Generator Input

```json
{
  "schemas": [
    {
      "aliases": null,
      "doc": null,
      "fields": [
        {
          "name": "x",
          "type": { "json": "\"float\"", "type": "float" }
        },
        {
          "name": "y",
          "type": { "json": "\"float\"", "type": "float" }
        }
      ],
      "file_path": "src/record_with_namespace.avsc",
      "fullname": "org.testorg.InternallyDefinedRecord",
      "json": "{\"type\":\"record\",\"namespace\":\"org.testorg\",\"name\":\"InternallyDefinedRecord\",\"fields\":[{\"name\":\"x\",\"type\":\"float\"},{\"name\":\"y\",\"type\":\"float\"}]}",
      "name": "InternallyDefinedRecord",
      "namespace": "org.testorg",
      "type": "record"
    },
  ],
  "package": {
    "name": "sample_schemas",
    "version": "0.1.0",
    "description": "An exmaple project that generates some basic types for rust and typescript"
  },
  "params": {
    "cargo_toml": true
  }
}
```

Note that the structure of the schema passed to the generator follows a slightly different schema from what is in the Avro specification.  This is to provide the needed information to the generator while keeping the structure of the schema data consistent for easier consumption by the generator and its templates.

### Generator Lua Environment

To render a template, the `generate.lua` script calls the `render` function, which is available as a global in the lua environment:

- `render(template, dest, params)` - renders `template` (path to template relative to the generator's `templates/` directory) and writes the output to a file located at `dest`, (path relative to the output directory.)  `params` can be populated with any parameters that should be passed to the template, and will be available in the template in the `params` variable.  (Params from the `params` argument will be merged into the generator params.)

To assist with code generation, the lua environment has been augmented with some additional functions

- `map` - Creates a map. A map differs from a Lua table in that it only accepts string keys making for more predictable serialization to a jason value when passed to tera templates
  - `map.update(dest, source)` - updates the `dest` map with keys from table `other`.  If keys in `dest` are present in `other`, they are overwritten.
  - `map.remove(m, key)` - removes and returns the value in map `m` at `key`
  - `map.keys(m)` - returns an array containing the keys of the map `m`
  - `map.values(m)` - returns an array containing the values of the map `m`
- `array` - Creates an array. An array differs from a Lua
  - `array.push(arr, val)` - adds the value `val` to the end of array `arr`
  - `array.append(arr, source)` - appends values from the array `source` to the end of the array `arr`
  - `array.map(arr, fn)` - returns an array containing values from `arr` after `fn` is applied to them
- `string.to_snake_case(s)` - converts an identifier string to snake case (e.g. `my_identifier_name`)
- `string.to_kebab_case(s)` - converts an identifier string to kebab case (e.g. `my-identifier-name`)
- `string.to_title_case(s)` - converts an identifier string to title case (e.g. `MyIdentifierName`)
- `string.to_camel_case(s)` - converts an identifier string to camel case (e.g. `myIdentifierName`)
- `string.to_const_case(s)` - converts an identifier string to const case (e.g. `MY_IDENTIFIER_NAME`)
- `string.split(s, sep)` - returns an array containing segments of the string `s` split on `sep`. (e.g. `string.split("hello world", " ")` returns `{ 1 = "hello", 2 = "world" }`).  `sep` is a regular expression that follows the same syntax as lua's `string.find` and `string.gusb` functions.

### Tera Template Environment

The tera template environment is provided the same `schemas`, `package`, and `params` variables, howevern the `params` variable also contains any parameters passed into the `render` function for the `params` argument.

The tera environment is augmented with the following filters:

- `snake_case`
- `kebab_case`
- `title_case`
- `camel_case`
- `const_case`

## License

With the exception of the content of the `examples` and `src/generators` directories, this project is licensed under the `GPL-3.0` license.

The content of the `examples` and `src/generators` directories can be copied, re-used, and modified without restriction.



