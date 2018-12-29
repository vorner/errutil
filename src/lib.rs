use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type Error = Box<dyn StdError + Send + Sync>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Context<M, E> {
    msg: M,
    error: E,
}

impl<M, E> Context<M, E> {
    pub fn new(msg: M, error: E) -> Self {
        Context {
            msg,
            error,
        }
    }
}

impl<M: Display, E> Display for Context<M, E> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        self.msg.fmt(fmt)
    }
}

impl<M: Debug + Display, E: Debug + StdError + 'static> StdError for Context<M, E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

pub trait ErrorExt {
    fn context<M: Display + Debug>(self, msg: M) -> Context<M, Self>
    where
        Self: Sized;

    // TODO: Convenience functions like iter_causes?
}

impl<E: StdError> ErrorExt for E {
    fn context<M: Debug + Display>(self, msg: M) -> Context<M, Self>
    where
        Self: Sized
    {
        Context::new(msg, self)
    }
}

pub trait ResultExt: Sized {
    type Err;
    type Ok;
    fn context<M: Display + Debug>(self, msg: M) -> Result<Self::Ok, Context<M, Self::Err>>;
    fn with_context<M: Display + Debug, F: FnOnce() -> M>(self, f: F)
        -> Result<Self::Ok, Context<M, Self::Err>>;
}

impl<T, E: StdError + Send + Sync + 'static> ResultExt for Result<T, E> {
    type Err = E;
    type Ok = T;
    fn context<M: Display + Debug>(self, msg: M) -> Result<Self::Ok, Context<M, Self::Err>> {
        self.with_context(|| msg)
    }
    fn with_context<M: Display + Debug, F: FnOnce() -> M>(self, f: F)
        -> Result<Self::Ok, Context<M, Self::Err>>
    {
        self.map_err(|e| e.context(f()))
    }
}
