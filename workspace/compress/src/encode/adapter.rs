use std::io::{Error, Write};

pub trait AdaptWrite<'a, In>: Adapter {
    /// A helper trait to allow introducing flexible constraints on the wrapped writer.
    ///
    /// This is normally where you would put constraints on `In` and implement the actual wrapping logic.
    fn wrap(self, internal: In) -> Result<Self::Write<'a>, Error>;
}

pub trait Adapter {
    type Write<'a>: Write + 'a;
}
