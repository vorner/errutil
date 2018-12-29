use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::sync::Mutex;

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
    fn sync_err(self) -> SyncError<Self>
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
    fn sync_err(self) -> SyncError<Self>
    where
        Self: Sized
    {
        SyncError::from(self)
    }
}

pub trait ResultExt: Sized {
    type Err;
    type Ok;
    fn context<M: Display + Debug>(self, msg: M) -> Result<Self::Ok, Context<M, Self::Err>>;
    fn with_context<M: Display + Debug, F: FnOnce() -> M>(self, f: F)
        -> Result<Self::Ok, Context<M, Self::Err>>;
    fn sync_err(self) -> Result<Self::Ok, SyncError<Self::Err>>;
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
    fn sync_err(self) -> Result<Self::Ok, SyncError<Self::Err>> {
        self.map_err(|e| e.sync_err())
    }
}

#[derive(Debug)]
pub struct SyncError<E>(Mutex<E>);

impl<E> From<E> for SyncError<E> {
    fn from(err: E) -> Self {
        SyncError(Mutex::new(err))
    }
}

impl<E: Display> Display for SyncError<E> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        self.0.lock().unwrap().fmt(fmt)
    }
}

impl<E: StdError> StdError for SyncError<E> { }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct MsgErr<D>(D);

impl<D: Display> Display for MsgErr<D> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        self.0.fmt(fmt)
    }
}

impl<D: Debug + Display> StdError for MsgErr<D> {}

pub fn err_msg<D: Debug + Display>(msg: D) -> impl StdError {
    MsgErr(msg)
}

#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($crate::err_msg($e).into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err($crate::err_msg(format!($fmt, $($arg)+)).into());
    };
}

#[macro_export(local_inner_macros)]
macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            bail!($e);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)+) => {
        if !($cond) {
            bail!($fmt, $($arg)+);
        }
    };
}

#[macro_export]
macro_rules! format_err {
    ($($arg:tt)*) => { $crate::err_msg(format!($arg)) }
}
