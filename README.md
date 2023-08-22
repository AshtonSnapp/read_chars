# read_chars

This is just a simple library I made for use in a new lexer for my programming language project [Rouge](https://github.com/AshtonSnapp/rouge). You are free to use it how you wish under the MIT license.

```rust
use read_chars::ReadChars;
use std::convert::From;
use std::io;
use std::fs::File;

fn main() -> io::Result<()> {
	let chars = ReadChars::from(File::open("test.txt")?);

	let max_nesting = chars.filter(|r| match r { Ok(c) if c == '(' || c == ')' => Some(c), _ => None })
		.scan(0, |acc, chr| if chr == '(' { acc + 1 } else { acc - 1 })
		.max()
		.ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;

	println!("Max parentheses nesting in file 'text,txt' is {max_nesting}.");

	Ok(())
}
```