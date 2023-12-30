#[cfg(test)]
mod tests {
 use blockwise_reader::FindPos;
 use blockwise_reader::PatternIdx;
 use stringreader::StringReader;

 use blockwise_reader::BlockWiseReader;
 use blockwise_reader::Error;

 #[test]
 fn test001() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!(0, bwr.available_bytes());
  assert_eq!(0, bwr.pos_get());
  assert_eq!(None, bwr.find(b'1'));
  assert_eq!(None, bwr.search(&[b'1']));
  assert_eq!("".as_bytes(), bwr.get());
  assert_eq!(3, bwr.slurp(1000)?);
  assert_eq!(Some(0), bwr.find(b'1'));
  assert_eq!(Some(0), bwr.search(&[b'1']));
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
  assert_eq!(4, bwr.size());
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
 fn test_slurp_loop() -> Result<(), std::io::Error> {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  assert_eq!(6, bwr.slurp_loop(1)?);
  assert_eq!(0, bwr.pos_get());
  assert_eq!(6, bwr.available_bytes());
  assert_eq!("123456".as_bytes(), bwr.get());
  Ok(())
 }

 #[test]
 fn test_practcal_example_001() -> Result<(), std::io::Error> {
  let sr = StringReader::new(
   r#"# Generated by NetworkManager
search localdomain
nameserver 8.8.8.8
"#,
  );

  let mut bwr = BlockWiseReader::new(Box::new(sr));

  assert!(bwr.slurp_match_repos("# Generated by NetworkManager\n".as_bytes())?);
  assert!(bwr.slurp_find_repos1(1024, b'\n')?);
  assert!(bwr.slurp_match_repos("nameserver ".as_bytes())?);
  let pos = bwr.pos_get();
  assert!(bwr.slurp_find_repos0(1024, b'\n')?);
  assert_eq!("8.8.8.8".as_bytes(), bwr.get_from_to_current(pos));

  Ok(())
 }

 #[test]
 fn test_00b() -> Result<(), Error> {
  for i in 1..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res = bwr.slurp_find_repos_loop(i, b'5', blockwise_reader::FindPos::Begin)?;
   assert!(res);
   assert_eq!(4, bwr.pos_get());
  }
  Ok(())
 }

 #[test]
 fn test_00b_3() -> Result<(), Error> {
  for i in 1..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res = bwr.slurp_find_repos_loop(i, b'5', blockwise_reader::FindPos::End)?;
   assert!(res);
   assert_eq!(5, bwr.pos_get());
  }
  Ok(())
 }

 #[test]
 fn test_00b_3_2() -> Result<(), Error> {
  for i in 1..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   assert_eq!(4, bwr.slurp(4)?);
   let res = bwr.slurp_find_repos_loop(i, b'5', blockwise_reader::FindPos::End)?;
   assert!(res);
   assert_eq!(5, bwr.pos_get());
  }
  Ok(())
 }

 #[test]
 fn test_00b_2() -> Result<(), Error> {
  for i in 1..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   bwr.pos_set(1);
   let res = bwr.slurp_find_repos_loop(i, b'9', blockwise_reader::FindPos::Begin)?;
   assert!(!res);
   assert_eq!(1, bwr.pos_get());
  }
  Ok(())
 }

 #[test]
 fn test_00c() {
  let sr = StringReader::new("123456");
  let mut bwr = BlockWiseReader::new(Box::new(sr));
  let res = bwr.slurp_find_repos_loop(0, b'5', blockwise_reader::FindPos::Begin);
  // assert_eq!(res, Err(Error::Msg("")));
  match res {
   Err(Error::Msg(x)) => assert_eq!(x, "buffersize 0 leads to an infinite loop"),
   _ => panic!(),
  }
 }

 #[test]
 fn test_00d() -> Result<(), Error> {
  for i in 3..10 {
   let sr = StringReader::new("123456789");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res = bwr.slurp_search_repos_loop(i, "67".as_bytes(), blockwise_reader::FindPos::Begin)?;
   assert!(res);
   assert_eq!(5, bwr.pos_get());
  }
  Ok(())
 }

 #[test]
 fn test_00d_2() -> Result<(), Error> {
  for i in 3..10 {
   let sr = StringReader::new("123456789");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res = bwr.slurp_search_repos_loop(i, "67".as_bytes(), blockwise_reader::FindPos::End)?;
   assert!(res);
   assert_eq!(7, bwr.pos_get());
  }
  Ok(())
 }

 fn slurp_find_multiple_repos_tests(cut: bool, fp: FindPos) -> Result<(), Error> {
  for i in 1..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res = bwr.slurp_find_multiple_repos(i, &[b'5', b'4'], cut, fp)?;

   match (i < 4, i < 5, cut, fp) {
    (true, _, _, _) => {
     assert!(!res);
     assert_eq!(0, bwr.pos_get());
    }
    (false, _, false, FindPos::Begin) => {
     assert!(res);
     assert_eq!(3, bwr.pos_get());
    }
    (false, _, false, FindPos::End) => {
     assert!(res);
     assert_eq!(4, bwr.pos_get());
    }
    (false, true, true, FindPos::Begin) => {
     assert!(res);
     assert_eq!(3, bwr.pos_get());
    }
    (_, false, true, FindPos::Begin) => {
     assert!(res);
     assert_eq!(4, bwr.pos_get());
    }
    (false, true, true, FindPos::End) => {
     assert!(res);
     assert_eq!(4, bwr.pos_get());
    }
    (_, false, true, FindPos::End) => {
     assert!(res);
     assert_eq!(5, bwr.pos_get());
    } // _ => panic!(),
   }
  }
  Ok(())
 }

 #[test]
 fn test_00e() -> Result<(), Error> {
  slurp_find_multiple_repos_tests(false, FindPos::Begin)
 }

 #[test]
 fn test_00e_3() -> Result<(), Error> {
  slurp_find_multiple_repos_tests(false, FindPos::End)
 }

 #[test]
 fn test_00e_2() -> Result<(), Error> {
  slurp_find_multiple_repos_tests(true, FindPos::Begin)
 }

 #[test]
 fn test_00e_4() -> Result<(), Error> {
  slurp_find_multiple_repos_tests(true, FindPos::End)
 }

 fn slurp_find_multiple_repos_idx_tests(cut: bool, fp: FindPos) -> Result<(), Error> {
  for i in 1..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res = bwr.slurp_find_multiple_repos_idx(i, &[b'5', b'4'], cut, fp)?;

   match (i < 4, i < 5, cut, fp) {
    (true, _, _, _) => {
     assert_eq!(None, res);
     assert_eq!(0, bwr.pos_get());
    }
    (false, _, false, FindPos::Begin) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(3, bwr.pos_get());
    }
    (false, _, false, FindPos::End) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(4, bwr.pos_get());
    }
    (false, true, true, FindPos::Begin) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(3, bwr.pos_get());
    }
    (_, false, true, FindPos::Begin) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(4, bwr.pos_get());
    }
    (false, true, true, FindPos::End) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(4, bwr.pos_get());
    }
    (_, false, true, FindPos::End) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(5, bwr.pos_get());
    } // _ => panic!(),
   }
  }
  Ok(())
 }

 #[test]
 fn test_slurp_find_multiple_repos_idx_001() -> Result<(), Error> {
  slurp_find_multiple_repos_idx_tests(false, FindPos::Begin)
 }

 #[test]
 fn test_slurp_find_multiple_repos_idx_002() -> Result<(), Error> {
  slurp_find_multiple_repos_idx_tests(false, FindPos::End)
 }

 #[test]
 fn test_slurp_find_multiple_repos_idx_003() -> Result<(), Error> {
  slurp_find_multiple_repos_idx_tests(true, FindPos::Begin)
 }

 #[test]
 fn test_slurp_find_multiple_repos_idx_004() -> Result<(), Error> {
  slurp_find_multiple_repos_idx_tests(true, FindPos::End)
 }

 fn slurp_search_multiple_repos_idx_tests(cut: bool, fp: FindPos) -> Result<(), Error> {
  for i in 1..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res =
    bwr.slurp_search_multiple_repos_idx(i, &["56".as_bytes(), "45".as_bytes()], cut, fp)?;

   println!("i:{i}");
   match (i < 5, i < 6, cut, fp) {
    (true, _, _, _) => {
     assert_eq!(None, res);
     assert_eq!(0, bwr.pos_get());
    }
    (false, _, false, FindPos::Begin) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(3, bwr.pos_get());
    }
    (false, _, false, FindPos::End) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(5, bwr.pos_get());
    }
    (false, true, true, FindPos::Begin) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(3, bwr.pos_get());
    }
    (_, false, true, FindPos::Begin) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(4, bwr.pos_get());
    }
    (false, true, true, FindPos::End) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(5, bwr.pos_get());
    }
    (_, false, true, FindPos::End) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(6, bwr.pos_get());
    } // _ => panic!(),
   }
  }
  Ok(())
 }

 #[test]
 fn test_slurp_search_multiple_repos_idx_001() -> Result<(), Error> {
  slurp_search_multiple_repos_idx_tests(false, FindPos::Begin)
 }

 #[test]
 fn test_slurp_search_multiple_repos_idx_002() -> Result<(), Error> {
  slurp_search_multiple_repos_idx_tests(false, FindPos::End)
 }

 #[test]
 fn test_slurp_search_multiple_repos_idx_003() -> Result<(), Error> {
  slurp_search_multiple_repos_idx_tests(true, FindPos::Begin)
 }

 #[test]
 fn test_slurp_search_multiple_repos_idx_004() -> Result<(), Error> {
  slurp_search_multiple_repos_idx_tests(true, FindPos::End)
 }

 fn slurp_search_multiple_repos_loop_idx_tests(cut: bool, fp: FindPos) {
  for i in 0..7 {
   let sr = StringReader::new("123456");
   let mut bwr = BlockWiseReader::new(Box::new(sr));
   let res =
    bwr.slurp_search_multiple_repos_loop_idx(i, &["56".as_bytes(), "456".as_bytes()], cut, fp);

   println!("i:{i}");
   match (i < 5, i < 6, cut, fp, res) {
    (_, _, _, _, Err(Error::Msg(err))) => {
     if i == 0 {
      assert_eq!("buffersize 0 leads to an infinite loop", err);
     } else if i <= 3 {
      assert_eq!("error: buffersize <= bytes.len()", err);
     } else {
      println!("err: {:?}", err);
      panic!();
     }
     // todo!()
    }
    (_, _, _, _, Err(Error::IO(_))) => {
     panic!()
    }
    (true, _, false, FindPos::Begin, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(3, bwr.pos_get());
    }
    (true, _, false, FindPos::End, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(6, bwr.pos_get());
    }
    (true, _, true, FindPos::Begin, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(4, bwr.pos_get());
    }
    (true, _, true, FindPos::End, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(6, bwr.pos_get());
    }
    (false, _, false, FindPos::Begin, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(3, bwr.pos_get());
    }
    (false, _, false, FindPos::End, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 1 }), res);
     assert_eq!(6, bwr.pos_get());
    }
    (false, true, true, FindPos::Begin, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(4, bwr.pos_get());
    }
    (_, false, true, FindPos::Begin, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(4, bwr.pos_get());
    }
    (false, true, true, FindPos::End, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(6, bwr.pos_get());
    }
    (_, false, true, FindPos::End, Ok(res)) => {
     assert_eq!(Some(PatternIdx { idx: 0 }), res);
     assert_eq!(6, bwr.pos_get());
    } // _ => panic!(),
   }
  }
 }

 #[test]
 fn test_slurp_search_multiple_repos_loop_idx_001() {
  slurp_search_multiple_repos_loop_idx_tests(false, FindPos::Begin)
 }

 #[test]
 fn test_slurp_search_multiple_repos_loop_idx_002() {
  slurp_search_multiple_repos_loop_idx_tests(false, FindPos::End)
 }

 #[test]
 fn test_slurp_search_multiple_repos_loop_idx_003() {
  slurp_search_multiple_repos_loop_idx_tests(true, FindPos::Begin)
 }

 #[test]
 fn test_slurp_search_multiple_repos_loop_idx_004() {
  slurp_search_multiple_repos_loop_idx_tests(true, FindPos::End)
 }
}
