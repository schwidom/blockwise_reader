#![allow(unused)]

/*!
The BlockWiseReader allows it to parse headers of files or streams where you
not exactly know how many bytes you need to read to be able to continue to parse.

So what you need is an educated guess for the amount you want to read.

The main task here is to avoid to read all the data before you begin to parse something.

Because there are cases where it is just too much.

For any token or sequence of tokens you want to find you can decide how many bytes you want to read ahead.
It can also be all of it if you are certain.

As soon as you have identified all parts you need, you can then continue to parse
your gathered bytes by more advanced parsers like for instance nom, combine, chumsky or pest.

```rust
use stringreader::StringReader;
use blockwise_reader::BlockWiseReader;

let sr = StringReader::new(
r#"# Generated by NetworkManager
search localdomain
nameserver 8.8.8.8
"#,
  );

let mut bwr = BlockWiseReader::new(Box::new(sr));

assert!(bwr.slurp_match_repos("# Generated by NetworkManager\n".as_bytes()).unwrap());
assert!(bwr.slurp_find_repos1(1024, b'\n').unwrap());
assert!(bwr.slurp_match_repos("nameserver ".as_bytes()).unwrap());
let pos = bwr.pos_get();
assert!(bwr.slurp_find_repos0(1024, b'\n').unwrap());
assert_eq!( "8.8.8.8".as_bytes(), bwr.get_from_to_current(pos));

```

It is also possible to search blockwise to a matching fixed byte slice. But there is the risk that this byte slice never will appear in the stream.
```rust
use stringreader::StringReader;
use blockwise_reader::BlockWiseReader;
use blockwise_reader::FindPos;

let sr = StringReader::new( r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit,
 sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
 Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip
 ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit
 esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat
 non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."#);

let mut bwr = BlockWiseReader::new(Box::new(sr));

// reads repeatedly 100 byte blocks and stops if match appears
assert!(bwr.slurp_search_repos_loop(100, "laborum".as_bytes(), FindPos::Begin).unwrap());
assert_eq!( 442, bwr.pos_get());

// ( Btw. When this value here changes over time this means the sourcecode formatter changed
// the code in the documentation, which is a bug in my opinion. The sourcecode formatter
// should not change things in strings and comments.)


```
*/

use memmem::{Searcher, TwoWaySearcher};
use std::{
 cmp::{max, min},
 hash::BuildHasher,
 io::Read,
 mem::swap,
};

/// this enum decides where to set the internal vector position after a search / find operation
#[derive(Clone, Copy)]
pub enum FindPos {
 /// set it to the begion of the search word or character
 Begin,
 /// set it to the end of the search word or character
 End,
}

/// The BlockWiseReader holds the data which are read in to a specific point and a reader to read from
pub struct BlockWiseReader<'a> {
 v: Vec<u8>,
 r: Box<dyn Read + 'a>,
 pos: usize,
 eof: bool,
}

#[derive(Debug)]
pub enum Error {
 IO(std::io::Error),
 Msg(&'static str),
}

impl From<std::io::Error> for Error {
 fn from(value: std::io::Error) -> Self {
  Self::IO(value)
 }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct PatternIdx {
 pub idx: usize,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct BufferIdx {
 idx: usize,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Finding {
 pi: PatternIdx,
 bi: BufferIdx,
}

impl<'a> Finding {
 fn min(&'a self, other: &'a Self) -> &'a Self {
  if self.bi <= other.bi {
   self
  } else {
   other
  }
 }
}

impl<'a> BlockWiseReader<'a> {
 /// creates a new BlockWiseReader from the given reader
 pub fn new(r: Box<dyn Read + 'a>) -> Self {
  Self {
   v: vec![],
   r,
   pos: 0,
   eof: false,
  }
 }

 /// bytes from the current pos position to the end of the internal vector
 pub fn available_bytes(&self) -> usize {
  self.v.len() - self.pos
 }

 /// overall size of the internal vector, is the same as available_bytes() + pos_get()
 pub fn size(&self) -> usize {
  self.v.len()
 }

 /// Reads bytecount bytes from the stream and returns the amount of available bytes starting at pos,
 /// which can be more than bytecount when subsequently already read more, but also less
 /// if there were not enough available.
 pub fn slurp(&mut self, bytecount: usize) -> Result<usize, std::io::Error> {
  let pos = self.pos;
  let read_start = self.v.len();
  if read_start >= pos + bytecount {
   return Ok(self.available_bytes());
  }
  let amount_to_read = pos + bytecount - read_start;
  self.v.resize(pos + bytecount, 0);
  let rod = self.r.read(&mut self.v[read_start..])?;
  if rod < amount_to_read {
   self.v.truncate(read_start + rod);
  }
  if rod == 0 {
   self.eof = true;
  }
  Ok(self.available_bytes())
 }

 /// Reads bytes from the stream in buffersize steps as long as there are bytes available.
 pub fn slurp_loop(&mut self, buffersize: usize) -> Result<usize, std::io::Error> {
  loop {
   let len = self.v.len();
   let newlen = len + buffersize;
   self.v.resize(newlen, 0);
   let rod = self.r.read(&mut self.v[len..])?;
   if rod < buffersize {
    self.v.truncate(len + rod);
   }
   if rod == 0 {
    self.eof = true;
    break;
   }
  }

  Ok(self.available_bytes())
 }

 /// searches a byte in the available bytes
 pub fn find(&self, e: u8) -> Option<usize> {
  self.v[self.pos..].iter().position(|x| x == &e)
 }

 /// searches a byte slice in the available bytes
 pub fn search(&self, bytes: &[u8]) -> Option<usize> {
  let search = TwoWaySearcher::new(bytes);
  search.search_in(&self.v[self.pos..])
 }

 /// sets the internal position
 pub fn pos_set(&mut self, pos: usize) {
  self.pos = pos;
 }

 /// adds to the internal position
 pub fn pos_add(&mut self, pos: usize) {
  self.pos += pos;
 }

 /// subtracts from the internal position
 pub fn pos_sub(&mut self, pos: usize) {
  self.pos -= pos;
 }

 /// adds to the internal position a positive or negative value
 pub fn pos_add_i(&mut self, pos: isize) {
  if pos >= 0 {
   self.pos_add(pos as usize)
  } else {
   self.pos_sub(pos as usize)
  }
 }

 /// removes all elements form the beginning of the internal vector to pos and returns the removed elements
 pub fn pos_cut(&mut self) -> Vec<u8> {
  let mut ret = self.v.split_off(self.pos);
  swap(&mut self.v, &mut ret);
  self.pos = 0;
  ret
 }

 /// copies the data of the slice s at the position pos
 pub fn pos_inject(&mut self, s: &[u8]) {
  let v3 = self.v.split_off(self.pos);
  self.v.extend(s);
  self.v.extend(v3);
 }

 /// returns all data from pos to the end of the internal vector
 pub fn get(&self) -> &[u8] {
  &self.v[self.pos..]
 }

 /// returns all data from pos - back to pos of the internal vector
 pub fn get_back(&self, back: usize) -> &[u8] {
  &self.v[self.pos - back..]
 }

 /// returns all data from the given pos to the end of the internal vector
 pub fn get_from(&self, pos: usize) -> &[u8] {
  &self.v[pos..]
 }

 /// returns all data from the given pos to the internal pos
 pub fn get_from_to_current(&self, pos: usize) -> &[u8] {
  &self.v[pos..self.pos]
 }

 /// slurps as much as the marker_str is long and returns true if the content is the same as the marker_str, repositions the current position to the end of the marker
 pub fn slurp_match_repos(&mut self, marker_str: &[u8]) -> Result<bool, std::io::Error> {
  let rod = self.slurp(marker_str.len())?;
  if rod < marker_str.len() {
   return Ok(false);
  }

  if marker_str != &self.get()[..marker_str.len()] {
   return Ok(false);
  }

  self.pos_add(marker_str.len());

  Ok(true)
 }

 /// matches a fixed string from pos - marker_str.len() to pos, returns true if matched
 pub fn match_back(&self, marker_str: &[u8]) -> bool {
  let len = marker_str.len();
  if len < self.pos {
   return false;
  }
  &self.get_back(len)[..len] == marker_str
 }

 /// Sets pos to find position + 1 if the byte was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 pub fn slurp_find_repos1(&mut self, bytecount: usize, e: u8) -> Result<bool, std::io::Error> {
  self.slurp_find_repos(bytecount, e, FindPos::End)
 }

 /// Sets pos to find position if the byte was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 pub fn slurp_find_repos0(&mut self, bytecount: usize, e: u8) -> Result<bool, std::io::Error> {
  self.slurp_find_repos(bytecount, e, FindPos::Begin)
 }

 /// Sets pos regarding the fp flag if the byte was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 pub fn slurp_find_repos(
  &mut self,
  bytecount: usize,
  e: u8,
  fp: FindPos,
 ) -> Result<bool, std::io::Error> {
  self.slurp(bytecount)?;
  Ok(match self.find(e) {
   None => false,
   Some(pos) => {
    let offset = match fp {
     FindPos::Begin => 0,
     FindPos::End => 1,
    };
    self.pos_add(pos + offset);
    true
   }
  })
 }

 /// Sets pos regarding the fp flag if one of the the bytes was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 /// Finds the nearest byte if cut is false.
 /// The parameter cut stops searching if a byte was found.
 pub fn slurp_find_multiple_repos(
  &mut self,
  bytecount: usize,
  se: &[u8],
  cut: bool,
  fp: FindPos,
 ) -> Result<bool, std::io::Error> {
  self.slurp(bytecount)?;
  let current_pos = self.pos_get();

  let mut foundpos: Option<usize> = None;
  // TODO optimization : shorter search if previously found something
  for e in se {
   if self.slurp_find_repos(bytecount, *e, FindPos::Begin)? {
    match foundpos {
     None => foundpos = Some(self.pos),
     Some(some_foundpos) => foundpos = Some(min(some_foundpos, self.pos)),
    }
    self.pos = current_pos;

    if cut {
     break;
    }
   }
  }

  match foundpos {
   None => Ok(false),
   Some(foundpos) => {
    match fp {
     FindPos::Begin => self.pos = foundpos,
     FindPos::End => self.pos = foundpos + 1,
    }
    Ok(true)
   }
  }
 }

 /// Sets pos regarding the fp flag if one of the the bytes was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 /// Finds the nearest byte if cut is false.
 /// The parameter cut stops searching if a byte was found.
 /// Returns the idx of the matched pattern.
 pub fn slurp_find_multiple_repos_idx(
  &mut self,
  bytecount: usize,
  se: &[u8],
  cut: bool,
  fp: FindPos,
 ) -> Result<Option<PatternIdx>, std::io::Error> {
  self.slurp(bytecount)?;
  let current_pos = self.pos_get();

  let mut foundpos: Option<Finding> = None;
  // TODO optimization : shorter search if previously found something
  for (idx, e) in se.iter().enumerate() {
   if self.slurp_find_repos(bytecount, *e, FindPos::Begin)? {
    let finding = Finding {
     pi: PatternIdx { idx },
     bi: BufferIdx { idx: self.pos },
    };
    match foundpos {
     None => foundpos = Some(finding),
     Some(some_foundpos) => foundpos = Some(*some_foundpos.min(&finding)),
    }
    self.pos = current_pos;

    if cut {
     break;
    }
   }
  }

  match foundpos {
   None => Ok(None),
   Some(foundpos) => {
    match fp {
     FindPos::Begin => self.pos = foundpos.bi.idx,
     FindPos::End => self.pos = foundpos.bi.idx + 1,
    }
    Ok(Some(foundpos.pi))
   }
  }
 }

 /// the current internal position value
 pub fn pos_get(&self) -> usize {
  self.pos
 }

 /// Sets pos to find position if the byte slice was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 pub fn slurp_search_repos0(
  &mut self,
  bytecount: usize,
  bytes: &[u8],
 ) -> Result<bool, std::io::Error> {
  self.slurp_search_repos(bytecount, bytes, FindPos::Begin)
 }

 /// Sets pos after find position if the byte slice was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 pub fn slurp_search_repos1(
  &mut self,
  bytecount: usize,
  bytes: &[u8],
 ) -> Result<bool, std::io::Error> {
  self.slurp_search_repos(bytecount, bytes, FindPos::End)
 }

 /// Sets pos regarding the fp flag find position if the byte slice was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 pub fn slurp_search_repos(
  &mut self,
  bytecount: usize,
  bytes: &[u8],
  fp: FindPos,
 ) -> Result<bool, std::io::Error> {
  self.slurp(bytecount);
  Ok(match self.search(bytes) {
   None => false,
   Some(pos) => {
    let offset = match fp {
     FindPos::Begin => 0,
     FindPos::End => bytes.len(),
    };
    self.pos_add(pos + offset);
    true
   }
  })
 }

 /// Sets pos regarding the fp flag if one of the the bytes was found in the available bytes.
 /// If nothing was found pos remains unaltered.
 /// Finds the nearest byte slice if cut is false.
 /// The parameter cut stops searching if a byte slice was found.
 /// Returns the idx of the matched pattern.
 pub fn slurp_search_multiple_repos_idx(
  &mut self,
  bytecount: usize,
  sbytes: &[&[u8]],
  cut: bool,
  fp: FindPos,
 ) -> Result<Option<PatternIdx>, std::io::Error> {
  self.slurp(bytecount)?;
  let current_pos = self.pos_get();

  let mut foundpos: Option<Finding> = None;
  // TODO optimization : shorter search if previously found something
  for (idx, bytes) in sbytes.iter().enumerate() {
   if self.slurp_search_repos(bytecount, *bytes, FindPos::Begin)? {
    let finding = Finding {
     pi: PatternIdx { idx },
     bi: BufferIdx { idx: self.pos },
    };
    match foundpos {
     None => foundpos = Some(finding),
     Some(some_foundpos) => foundpos = Some(*some_foundpos.min(&finding)),
    }
    self.pos = current_pos;

    if cut {
     break;
    }
   }
  }

  match foundpos {
   None => Ok(None),
   Some(foundpos) => {
    match fp {
     FindPos::Begin => self.pos = foundpos.bi.idx,
     FindPos::End => self.pos = foundpos.bi.idx + sbytes[foundpos.pi.idx].len(),
    }
    Ok(Some(foundpos.pi))
   }
  }
 }

 /// Reads bytes from the stream in buffersize steps as long as there are bytes available to the point where the char was found.
 pub fn slurp_find_repos_loop(
  &mut self,
  buffersize: usize,
  e: u8,
  fp: FindPos,
 ) -> Result<bool, Error> {
  if 0 == buffersize {
   return Err(Error::Msg("buffersize 0 leads to an infinite loop"));
  }
  let oldpos = self.pos;
  loop {
   if self.slurp_find_repos(buffersize, e, fp)? {
    return Ok(true);
   } else {
    if self.eof {
     self.pos = oldpos;
     return Ok(false);
    } else {
     self.pos_add(self.available_bytes());
    }
   }
  }
 }

 /// Reads bytes from the stream in buffersize steps as long as there are bytes available to the point where the byte slice was found.
 /// The buffer must be bigger than the byte slice.
 pub fn slurp_search_repos_loop(
  &mut self,
  buffersize: usize,
  bytes: &[u8],
  fp: FindPos,
 ) -> Result<bool, Error> {
  if 0 == buffersize {
   return Err(Error::Msg("buffersize 0 leads to an infinite loop"));
  }
  if buffersize <= bytes.len() {
   return Err(Error::Msg("error: buffersize <= bytes.len()"));
  }
  let oldpos = self.pos;
  let mut offset = 0;
  loop {
   if self.slurp_search_repos(buffersize + offset, bytes, fp)? {
    return Ok(true);
   } else {
    if self.eof {
     self.pos = oldpos;
     return Ok(false);
    } else {
     self.pos_add(self.available_bytes());
     offset = bytes.len() - 1;
     self.pos_sub(offset);
    }
   }
  }
 }

 pub fn slurp_search_multiple_repos_loop_idx(
  &mut self,
  buffersize: usize,
  sbytes: &[&[u8]],
  cut: bool,
  fp: FindPos,
 ) -> Result<Option<PatternIdx>, Error> {
  if 0 == buffersize {
   return Err(Error::Msg("buffersize 0 leads to an infinite loop"));
  }
  let mut max_bytes_len: usize = 0;
  for bytes in sbytes {
   max_bytes_len = max(max_bytes_len, bytes.len());
   if buffersize <= bytes.len() {
    return Err(Error::Msg("error: buffersize <= bytes.len()"));
   }
  }
  let oldpos = self.pos;
  let mut offset = 0;
  loop {
   if let Some(pattern_idx) =
    self.slurp_search_multiple_repos_idx(buffersize + offset, sbytes, cut, fp)?
   {
    return Ok(Some(pattern_idx));
   } else {
    if self.eof {
     self.pos = oldpos;
     return Ok(None);
    } else {
     self.pos_add(self.available_bytes());
     offset = max_bytes_len - 1;
     self.pos_sub(offset);
    }
   }
  }
 }

 /// returns true if eof is reached
 fn eof(&self) -> bool {
  self.eof
 }
}
