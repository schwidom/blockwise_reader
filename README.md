# blockwise_reader
Reading and pre-parsing of large files or streams.

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

assert!(bwr.slurp_search_repos_loop(1024, "laborum".as_bytes(), FindPos::Begin).unwrap());
assert_eq!( 447, bwr.pos_get());

```

