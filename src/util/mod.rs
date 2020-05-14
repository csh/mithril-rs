use bytes::Buf;

pub trait RunescapeBuf: Buf {
    #[inline]
    fn get_rs_string(&mut self) -> String {
        let mut result = String::default();
        loop {
            match self.get_u8() {
                10 => break,
                c => result.push(char::from(c)),
            }
        }
        result
    }
}

impl<B: Buf> RunescapeBuf for B {}
