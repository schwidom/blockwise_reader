#[cfg(test)]
mod tests {
 use stringreader::StringReader;

 use blockwise_reader::BlockWiseReader;

 #[test]
 fn test001() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!(0, bwr.available_bytes());
  assert_eq!(0, bwr.pos_get());
  assert_eq!("".as_bytes(), bwr.get());
  assert_eq!(3, bwr.slurp(1000)?);
  assert_eq!(3, bwr.available_bytes());
  assert_eq!(0, bwr.pos_get());
  assert_eq!(3, bwr.size());
  assert_eq!("123".as_bytes(), bwr.get());
  assert_eq!(3, bwr.slurp(1000)?);
  assert_eq!("123".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test002() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!("".as_bytes(), bwr.get());
  assert_eq!(3, bwr.slurp(3)?);
  assert_eq!(3, bwr.available_bytes());
  assert_eq!(0, bwr.pos_get());
  assert_eq!("123".as_bytes(), bwr.get());
  assert_eq!(3, bwr.slurp(3)?);
  assert_eq!("123".as_bytes(), bwr.get());
  assert_eq!(3, bwr.slurp(0)?);
  assert_eq!("123".as_bytes(), bwr.get());
  assert_eq!(4, bwr.slurp(4)?);
  assert_eq!("1234".as_bytes(), bwr.get());
  bwr.pos_add(2);
  assert_eq!(2, bwr.available_bytes());
  assert_eq!( 4, bwr.size());
  assert_eq!(2, bwr.pos_get());
  assert_eq!("1234".as_bytes(), bwr.get_back(2));
  assert_eq!(4, bwr.slurp(4)?);
  assert_eq!("3456".as_bytes(), bwr.get());
  bwr.pos_add(2);
  assert_eq!(2, bwr.available_bytes());
  assert_eq!(4, bwr.pos_get());
  assert_eq!("56".as_bytes(), bwr.get());
  bwr.pos_add(2);
  assert_eq!("".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test003() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert!(bwr.slurp_match_repos(&"123".as_bytes())?);
  assert!(bwr.slurp_match_repos(&"456".as_bytes())?);
  assert!(bwr.slurp_match_repos(&"".as_bytes())?);
  assert!(!bwr.slurp_match_repos(&"x".as_bytes())?);
  Ok(())
 }

 #[test]
 fn test004() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert!(!bwr.slurp_find_repos1(1000, b'9')?);
  assert!(bwr.slurp_find_repos1(1000, b'3')?);
  assert_eq!("456".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test005() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert!(bwr.slurp_find_repos1(1000, b'3')?);
  assert!(!bwr.slurp_find_repos1(1000, b'9')?);
  assert_eq!("456".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test006() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert!(bwr.slurp_find_repos0(1000, b'3')?);
  assert_eq!("3456".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test007() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!(None, bwr.search("34".as_bytes()));
  assert!(bwr.slurp_search_repos1(1000, "34".as_bytes())?);
  assert_eq!(Some(0), bwr.search("56".as_bytes()));
  assert_eq!("56".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test008() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!(None, bwr.search("34".as_bytes()));
  assert!(bwr.slurp_search_repos0(1000, "34".as_bytes())?);
  assert_eq!(Some(0), bwr.search("34".as_bytes()));
  assert_eq!("3456".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test009() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!(6, bwr.slurp(6)?);
  assert!(bwr.slurp_search_repos1(1000, "123".as_bytes())?);
  assert_eq!(3, bwr.pos_get());
  assert_eq!("456".as_bytes(), bwr.get());
  assert_eq!("123".as_bytes(), bwr.pos_cut());
  assert_eq!(0, bwr.pos_get());
  assert_eq!("456".as_bytes(), bwr.get());
  bwr.pos_set(1);
  bwr.pos_inject("abc".as_bytes());
  bwr.pos_set(0);
  assert_eq!("4abc56".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test00a() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!(6, bwr.slurp_loop(1)?);
  assert_eq!(0, bwr.pos_get());
  assert_eq!(6, bwr.available_bytes());
  assert_eq!("123456".as_bytes(), bwr.get());
  Ok(())
 }
}
