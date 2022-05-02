// The contents of this file is a partial copy of
// https://github.com/rust-lang/rust/blob/508e0584e384556b7e66f57b62e4feeba864b6da/src/test/rustdoc/higher-ranked-trait-bounds.rs
// which is licensed under
// https://github.com/rust-lang/rust/blob/master/LICENSE-MIT. The license is as
// follows:
//
//   Permission is hereby granted, free of charge, to any person obtaining a copy
//   of this software and associated documentation files (the "Software"), to deal
//   in the Software without restriction, including without limitation the rights
//   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//   copies of the Software, and to permit persons to whom the Software is
//   furnished to do so, subject to the following conditions:
//
//   The above copyright notice and this permission notice shall be included in
//   all copies or substantial portions of the Software.
//
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//   SOFTWARE.

// @has foo/fn.test3.html
// @has - '//pre' "pub fn test3<F>() where F: for<'a, 'b> Fn(&'a u8, &'b u8),"
pub fn test3<F>()
where
    F: for<'a, 'b> Fn(&'a u8, &'b u8),
{
}

// @has foo/struct.Foo.html
pub struct Foo<'a> {
    pub some_func: for<'c> fn(val: &'c i32) -> i32,
}

// @has - '//span[@id="structfield.some_func"]' "some_func: for<'c> fn(val: &'c i32) -> i32"
