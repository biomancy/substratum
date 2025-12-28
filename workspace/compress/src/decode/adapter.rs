use std::io::{BufRead, Error, Read};

pub trait AdaptRead<'a, In>: Adapter {
    /// A helper trait to allow introducing flexible constraints on the wrapped reader.
    ///
    /// This is normally where you would put constraints on `In` and implement the actual wrapping logic.
    fn wrap(self, internal: In) -> Result<Self::Read<'a>, Error>;
}

pub trait AdaptBufRead<'a, In>: Adapter {
    /// A helper trait to allow introducing flexible constraints on the wrapped buffered reader.
    ///
    /// This is normally where you would put constraints on `In` and implement the actual wrapping logic.
    fn wrap(self, internal: In) -> Result<Self::BufRead<'a>, Error>;
}

pub trait Adapter {
    type Read<'a>: Read + 'a;
    type BufRead<'a>: BufRead + 'a;
}
