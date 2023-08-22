use std::{
	convert::From,
	io,
	io::{
		Bytes,
		Read,
	},
	iter::Iterator,
	str,
};

pub struct ReadChars<R: Read> {
	inner: Bytes<R>,
}

impl<R: Read> From<R> for ReadChars<R> {
	fn from(value: R) -> Self {
		ReadChars {
			inner: value.bytes(),
		}
	}
}

impl<R: Read> Iterator for ReadChars<R> {
	type Item = io::Result<char>;
	
	fn next(&mut self) -> Option<Self::Item> {
		let mut buf = [0u8; 4];

		buf[0] = match self.inner.next()? {
			Ok(byte) => byte,
			Err(e) => return Some(Err(e)),
		};

		let need_bytes = {
			let start = buf[0];

			if start >> 7 == 0b00000 { 0 } else
			if start >> 5 == 0b00110 { 1 } else
			if start >> 4 == 0b01110 { 2 } else
			if start >> 3 == 0b11110 { 3 } else {
				return Some(Err(io::Error::from(io::ErrorKind::InvalidData)))
			}
		};

		if need_bytes != 0 {
			for idx in 1..=need_bytes {
				buf[idx] = match self.inner.next()? {
					Ok(byte) if byte >> 6 == 0b10 => byte,
					Ok(_) => return Some(Err(io::Error::from(io::ErrorKind::InvalidData))),
					Err(e) => return Some(Err(e))
				}
			}
		}

		let chr = match str::from_utf8(&buf[0..=need_bytes]) {
			Ok(s) => s.chars().next()?,
			Err(e) => return Some(Err(io::Error::new(io::ErrorKind::InvalidData, e))),
		};

		Some(Ok(chr))
	}
}