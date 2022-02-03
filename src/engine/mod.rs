pub mod kvstore;
pub mod sled;
use either::Either;
use failure::Error;

pub trait KvsEngine {
    // type Result<T> = Result<T, Self::Error>;
    fn set(&mut self, key: String, value: String) -> Result<(), Error>;
    fn get(&mut self, key: String) -> Result<Option<String>, Error>;
    fn remove(&mut self, key: String) -> Result<(), Error>;
}

impl<L, R> KvsEngine for Either<L, R>
where
    L: KvsEngine,
    R: KvsEngine,
{
    fn set(&mut self, key: String, value: String) -> Result<(), Error> {
        match self {
            Either::Left(l) => l.set(key, value),
            Either::Right(r) => r.set(key, value),
        }
    }

    fn get(&mut self, key: String) -> Result<Option<String>, Error> {
        match self {
            Either::Left(l) => l.get(key),
            Either::Right(r) => r.get(key),
        }
    }

    fn remove(&mut self, key: String) -> Result<(), Error> {
        match self {
            Either::Left(l) => l.remove(key),
            Either::Right(r) => r.remove(key),
        }
    }
}
