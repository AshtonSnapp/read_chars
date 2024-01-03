//! [ReadChars] is a simple iterator crate for turning the bytes of an I/O reader into characters
//! for use by things like handwritten lexers.

//--> Imports <--

use std::{
    collections::VecDeque,
	convert::From,
	default::Default,
	io,
	io::{
		Bytes,
		Read,
	},
	iter::Iterator,
	ops::Range,
	str,
};

//--> Structs <--

/// An iterator that takes any type implementing [std::io::Read] and iterates over the UTF-8
/// characters in it. It returns an io::Result with the success value being a tuple of the character and its byte span.
///
/// It also contains an internal queue so it doesn't have to read bytes in every time it iterates.
pub struct ReadChars<R: Read> {
	inner: Bytes<R>,
	current: usize,
    queue: VecDeque<io::Result<(char, Range<usize>)>>,
}

//--> Functions <--

impl<R: Read> ReadChars<R> {
	/// The default capacity of the internal queue.
	pub const DEFAULT_CAPACITY: usize = 16;

	/// A constructor that allows you to specify a custom queue capacity.
	/// If you are using the default capacity, it's easier to use [From].
	pub fn new(reader: R, capacity: usize) -> ReadChars<R> {
		ReadChars {
			inner: reader.bytes(),
			current: usize::default(),
			queue: VecDeque::with_capacity(capacity),
		}
	}
}

impl<R: Read> From<R> for ReadChars<R> {
	fn from(value: R) -> Self {
		ReadChars {
			inner: value.bytes(),
			current: usize::default(),
            queue: VecDeque::with_capacity(ReadChars::<R>::DEFAULT_CAPACITY),
		}
	}
}

impl<R: Read> Iterator for ReadChars<R> {
	type Item = io::Result<(char, Range<usize>)>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.queue.is_empty() { 'outer: while self.queue.len() != self.queue.capacity() {
			let mut buffer = [0x0u8; 4];
			let chr_start = self.current;

			buffer[0] = match self.inner.next() {
				Some(Ok(c)) => c,
				Some(Err(e)) => return Some(Err(e)),
				None => return None,
			};
			self.current += 1;

			let need_bytes = {
				let start = buffer[0];

				if start.is_ascii() { 0 } else
				if start >> 5 == 0b110 { 1 } else
				if start >> 4 == 0b1110 { 2 } else
				if start >> 3 == 0b11110 { 3 } else
				{
					self.queue.push_front(Err(io::Error::from(io::ErrorKind::InvalidData)));
					self.current += 1;
					continue;
				}
			};

			if need_bytes != 0 {
				for idx in 1..=need_bytes {
					buffer[idx] = match self.inner.next() {
						Some(Ok(byte)) if byte >> 6 == 0b10 => {
							self.current += 1;
							byte
						},
						Some(Ok(_)) => {
							self.queue.push_front(Err(io::Error::from(io::ErrorKind::InvalidData)));
							self.current += 1;
							continue 'outer;
						},
						Some(Err(e)) => {
							self.queue.push_front(Err(e));
							self.current += 1;
							continue 'outer;
						},
						None => {
							self.queue.push_front(Err(io::Error::from(io::ErrorKind::UnexpectedEof)));
							self.current += 1;
							continue 'outer;
						}
					};
				}
			}

			let chr = match str::from_utf8(&buffer[0..=need_bytes]) {
				Ok(chr) => chr.chars().next().unwrap(),
				Err(e) => {
					self.queue.push_front(Err(io::Error::new(io::ErrorKind::InvalidData, e)));
					continue 'outer;
				},
			};

			self.queue.push_front(Ok((chr, chr_start..self.current)));
		}}

		self.queue.pop_back()
	}
}