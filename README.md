# cbc with Rust

cbc is a subset of c language.
It is based on [ふつうのコンパイラをつくろう](https://www.amazon.co.jp/%E3%81%B5%E3%81%A4%E3%81%86%E3%81%AE%E3%82%B3%E3%83%B3%E3%83%91%E3%82%A4%E3%83%A9%E3%82%92%E3%81%A4%E3%81%8F%E3%82%8D%E3%81%86-%E9%9D%92%E6%9C%A8-%E5%B3%B0%E9%83%8E/dp/4797337958).
In this book, CBC is implemented using Java. But I rewrite it with Rust.

## Todo

- [ ] Checks if operand is an array or pointer: `1[0]`
- [ ] Chekcs Checks whether the type of the operand is a structure or union with memb: `1.memb`
- [ ] Checks whether the type of the operand is a pointer to a structure or union with memb: `1->memb`
- [ ] Checks if operand is an array or pointer:: `*1`
- [ ] Checks if an operand is assignable: `&1`
- [ ] Checks if an operand is assignable: `++1`
- [ ] type checks
