//! [ReadChars] is a simple iterator crate for turning the bytes of an I/O reader into characters
//! for use by things like handwritten lexers.

use std::{
    collections::VecDeque,
	convert::From,
	io,
	io::{
		Bytes,
		Read,
	},
	iter::Iterator,
	str,
};

/// An iterator that takes any type implementing [std::io::Read] and loops over the UTF-8
/// characters in it.
///
/// It also contains an internal queue so it doesn't have to read bytes in every time it iterates.
pub struct ReadChars<R: Read> {
	inner: Bytes<R>,
    queue: VecDeque<char>,
}

impl<R: Read> From<R> for ReadChars<R> {
	fn from(value: R) -> Self {
		ReadChars {
			inner: value.bytes(),
            queue: VecDeque::with_capacity(16),
		}
	}
}

impl<R: Read> Iterator for ReadChars<R> {
	type Item = io::Result<char>;
	
	fn next(&mut self) -> Option<Self::Item> {
        if self.queue.is_empty() {
            while self.queue.len() < 16 {
                let mut buf = [0u8; 4];

                buf[0] = match self.inner.next() {
                    Some(Ok(b)) => b,
                    Some(Err(e)) => return Some(Err(e)),
                    None => break,
                };

                let need_bytes = {
                    let start = buf[0];

                    if start.is_ascii() { 0 } else
                    if start >> 5 == 0b110 { 1 } else
                    if start >> 4 == 0b1110 { 2 } else
                    if start >> 3 == 0b11110 { 3 } else {
                        return Some(Err(io::Error::from(io::ErrorKind::InvalidData)))
                    }
                };

                if need_bytes != 0 {
                    for idx in 1..=need_bytes {
                        buf[idx] = match self.inner.next() {
                            Some(Ok(b)) if b >> 6 == 0b10 => b,
                            Some(Ok(_)) => return Some(Err(io::Error::from(io::ErrorKind::InvalidData))),
                            Some(Err(e)) => return Some(Err(e)),
                            None => return Some(Err(io::Error::from(io::ErrorKind::InvalidData))),
                        };
                    }
                }

                let chr = match str::from_utf8(&buf[0..=need_bytes]) {
                    Ok(c) => chr,
                    Err(e) => return Some(Err(io::Error::new(io::ErrorKind::InvalidData, e))),
                };

                queue.push_back(chr);
        }

        Some(Ok(queue.pop()?))
	}
}
