# fsm

a file utility for storing additional data for files and directories. storing
tags, comments, and collections of files and directories.

## Setup

this works in a similar way as `git` in that a specified parent directory will
contain information for all files and subdirectories. when making changes by
adding tags, comments, or updating collections the tool will look for the root
directory from the current working directory.

### Initialization

you can initalize a directory by running:

```
fsm db init
```

by default the db will be initalized in a `json` format but can be changed by
specifying `--format` and the values can be:

1. `json` - the default value if not specified
2. `json-pretty` - writes json data in a more friendly and readable format
3. `binary` - writes binary data

```
fsm db init --format binary
```

## Tags and Comments

you are able to assign various tags to files or directories from the root of
the `.fsm` directory.

```
fsm set -t source:http://example.com/id/1234 -t important ./myfile.txt
```

this will assign the tags `source` and `important` to `myfile.txt` in the
current working directory. when a tag is given a value like
`http://example.com/id/1234` it will attempt to parse the value as if its a url
before defaulting to just saving it as a string in the database.

the order in which tags values will be parsed:
1. `i64` - any string value that can be full parsed to a 64 bit signed integer
2. `bool` - a string value that is `true` or `false`
3. `Url` - a string value that is in a valid `URL` format
4. `string` - fallback to store as a `UTF-8` string

to remove a previously set tag:

```
fsm set -d important ./myfile.txt
```

similar with tags, comments can be applied to any directory or file. there is
no special parsing performed on the comment string and will just store them.

```
fsm set -c "these files need to be reviewed" ./myfile.txt ./config.json
```

this will assign the comment to all the files specified creating or updating
any previously set value.

to remove a prevously set comment:

```
fsm set --drop-comment ./config.json
```

## Collections

collections allow you do group files/directories together that is outside of
the normal file system structure.

creating a new collection:

```
fsm coll create my_collection
```

adding items to the collection

```
fsm coll push ./myfile.txt ./config.json ./dir/notes.txt ./subdir/dir/stuff.docx
```

removing items from the collection

```
fsm coll pop ./config.json ./dir/notes.txt
```

removing a collection

```
fsm coll delete my_collection
```

## Opening

certain tags can be opened up by the tool such as urls to a specific website.
you can open up files for a collection in the default application specified
by the system. example of opening up a url attached to a file:

```
fsm set -t source:https://example.com/id/1234 --self
fsm open -t source --self
```

this will open up the `source` tag url in a browser that is attached to the
database itself.

## Supported Systems

currently the tool has been tested on windows and will more than likely work
on other operating systems. the general idea is for it to be agnostic and
implement specific system behavior when necessary.

more testing will need to be done to work out kinks.
