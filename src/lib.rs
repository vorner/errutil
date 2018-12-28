use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use backtrace::Backtrace;
pub use err_derive::Error;

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

impl<M: Display + Debug, E: Debug + StdError + 'static> StdError for Context<M, E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

pub trait ErrorExt: Sized {
    fn context<M: Display + Debug>(self, msg: M) -> Context<M, Self>;
    fn with_backtrace(self) -> WithBacktrace;

    // TODO: Convenience functions like iter_causes?
}

impl<E: StdError + Send + Sync + Sized + 'static> ErrorExt for E {
    fn context<M: Display + Debug>(self, msg: M) -> Context<M, Self> {
        Context::new(msg, self)
    }
    fn with_backtrace(self) -> WithBacktrace {
        WithBacktrace::from(Error::from(self))
    }
}

pub trait ResultExt: Sized {
    type Err;
    type Ok;
    fn context<M: Display + Debug>(self, msg: M) -> Result<Self::Ok, Context<M, Self::Err>>;
    fn with_context<M: Display + Debug, F: FnOnce() -> M>(self, f: F)
        -> Result<Self::Ok, Context<M, Self::Err>>;
    fn with_backtrace(self) -> Result<Self::Ok, WithBacktrace>;
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
    fn with_backtrace(self) -> Result<Self::Ok, WithBacktrace> {
        self.map_err(ErrorExt::with_backtrace)
    }
}

#[derive(Debug)]
pub struct WithBacktrace {
    err: Error,
    backtrace: Option<Backtrace>,
}

impl WithBacktrace {
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.backtrace
            .as_ref()
            .or_else(|| Self::find_bt(&*self.err).and_then(WithBacktrace::backtrace))
    }
    pub fn find_bt<'a>(mut e: &'a (dyn StdError + 'static)) -> Option<&'a WithBacktrace> {
        while let Some(source) = e.source() {
            if let Some(re) = source.downcast_ref::<WithBacktrace>() {
                return Some(re);
            }
            e = source;
        }
        None
    }

    pub fn is<T: StdError + 'static>(&self) -> bool {
        self.err.is::<T>()
    }
    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        self.err.downcast_ref::<T>()
    }
    pub fn downcast_mut<T: StdError + 'static>(&mut self) -> Option<&mut T> {
        self.err.downcast_mut::<T>()
    }
}

impl Display for WithBacktrace {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        Display::fmt(&self.err, fmt)
    }
}

impl StdError for WithBacktrace {
    fn description(&self) -> &str {
        self.err.description()
    }
    fn cause(&self) -> Option<&dyn StdError> {
        // Yes, we are just trying to play nice with old error types and forward this method too.
        #[allow(deprecated)]
        self.err.cause()
    }
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.err.source()
    }
}

impl From<Error> for WithBacktrace {
    fn from(err: Error) -> Self {
        // Look for the deepest backtrace if there's one already
        let backtrace = if Self::find_bt(&*err).is_some() {
            None
        } else {
            Some(Backtrace::new())
        };
        Self {
            err,
            backtrace,
        }
    }
}
