# Simplified Tab Notation

Simplified tab notation is an attempt at making ASCII guitar tabs easier to write. Instead of writing out all of the tab lines by hand, this library will automate the process for you. The syntax is simple to use and makes the workflow of guitar tab writing a breeze.

## Options

Options can be written within square brackets. Each option will be separated by a semicolon `;`. Each individual option will be made up of an option name and a value separated by an equals `=` sign.

### Names and Values

- time - can be set to any time signature in the format of `n/n` where `n` is any whole integer number; defaults to `4/4` if not set.
- fidelity - can be set to any whole integer number; defaults to `16` if not set.

### Examples

```
[time=6/8; fidelity=8]
```

## Symbols

- `[A-G][b#]?` : note literal - represents a note within the standard note range of A to G and can be modified with a flat 'b' or sharp '#' symbol.
- `[0-9]+` : number literal - represents any whole integer number from 0 to 9 and can be one or more digits long.
- `.` : empty space operator - represents a blank space in the guitar tabs when nothing is being played.
- `,` : next beat operator - represents a command to add empty spaces until the next beat is reached.
- `:[0-9]+` : empty space spread operator - represents blank spaces to be added for the provided number of times following the `:` operator.
- `;[0-9]+` : next beat spread operator - represents commands to add empty spaces until the next beat after the specified amount following the `;` is reached.

### Examples

```
C F A# D# G C
0 2,
0 2,
0 2,
0 2,
;4
. 3 5,
. 3 5,
. 3 5,
. 3 5,
;4
:2 5 7,
:2 5 7,
:2 5 7,
:2 5 7,
:2 7,
:2 9,
:2 7,
. 10,
0 7 9,
```