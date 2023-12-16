#![allow(unused)]

use memmem::{Searcher, TwoWaySearcher};
use std::{io::Read, mem::swap};

pub struct BlockWiseReader<'a> {
 v: Vec<u8>,
 r: Box<dyn Read + 'a>,
 pos: usize,
}

impl<'a> BlockWiseReader<'a> {
 pub fn new(r: Box<dyn Read + 'a>) -> Self {
  Self {
   v: vec![],
   r,
   pos: 0,
  }
 }

 /// bytes from the current pos position to the end of the internal vector
 pub fn available_bytes(&self) -> usize {
  self.v.len() - self.pos
 }

 /// overall size of the internal vector, is the same as available_bytes() + pos_get()
 pub fn size( &self) -> usize {
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
    break;
   }
  }

  Ok(self.available_bytes())
 }

 pub fn find(&self, e: u8) -> Option<usize> {
  self.v[self.pos..].iter().position(|x| x == &e)
 }

 pub fn search(&self, bytes: &[u8]) -> Option<usize> {
  let search = TwoWaySearcher::new(bytes);
  search.search_in(&self.v[self.pos..])
 }

 pub fn pos_set(&mut self, pos: usize) {
  self.pos = pos;
 }

 pub fn pos_add(&mut self, pos: usize) {
  self.pos += pos;
 }

 pub fn pos_sub(&mut self, pos: usize) {
  self.pos -= pos;
 }

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

 /// sets pos to find position position + 1 if found
 pub fn slurp_find_repos1(&mut self, bytecount: usize, e: u8) -> Result<bool, std::io::Error> {
  self.slurp(bytecount)?;
  Ok(match self.find(e) {
   None => false,
   Some(pos) => {
    self.pos_add(pos + 1);
    true
   }
  })
 }

 /// sets pos to find position if found
 pub fn slurp_find_repos0(&mut self, bytecount: usize, e: u8) -> Result<bool, std::io::Error> {
  self.slurp(bytecount)?;
  Ok(match self.find(e) {
   None => false,
   Some(pos) => {
    self.pos_add(pos);
    true
   }
  })
 }

 /// the current pos value
 pub fn pos_get(&self) -> usize {
  self.pos
 }

 /// sets pos to find position if found
 pub fn slurp_search_repos0(
  &mut self,
  bytecount: usize,
  bytes: &[u8],
 ) -> Result<bool, std::io::Error> {
  self.slurp(bytecount);
  Ok(match self.search(bytes) {
   None => false,
   Some(pos) => {
    self.pos_add(pos);
    true
   }
  })
 }

 /// sets pos after find position if found
 pub fn slurp_search_repos1(
  &mut self,
  bytecount: usize,
  bytes: &[u8],
 ) -> Result<bool, std::io::Error> {
  self.slurp(bytecount);
  Ok(match self.search(bytes) {
   None => false,
   Some(pos) => {
    self.pos_add(pos + bytes.len());
    true
   }
  })
 }
}
