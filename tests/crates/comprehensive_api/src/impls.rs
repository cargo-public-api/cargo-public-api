use crate::structs::Plain;

impl Plain {
    pub fn new() -> Plain {
        Plain { x: 4 }
    }

    pub fn f() {}

    pub fn s1(self) {}

    pub fn s2(&self) {}

    pub fn s3(&mut self) {}
}
