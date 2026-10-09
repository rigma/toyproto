use super::{CLOCK_ID_MASK, Tid, errors::NextTidError};
use chacha20::ChaCha20Rng;
use rand::Rng;
use std::{
    sync::Mutex,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

pub trait TidClock {
    type Error;

    fn next_tid(&self) -> Result<Tid, Self::Error>;
}

pub(super) struct DefaultClock(Mutex<Option<(u64, u16)>>);

impl DefaultClock {
    #[inline]
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }
}

impl TidClock for DefaultClock {
    type Error = NextTidError;

    fn next_tid(&self) -> Result<Tid, Self::Error> {
        let mut last_tid = self.0.lock()?;
        let now = current_timestamp()?;

        let (timestamp, clock_id) = match *last_tid {
            Some((last_timestamp, clock_id)) if now <= last_timestamp => {
                (last_timestamp + 1, clock_id)
            }
            Some((_, clock_id)) => (now, clock_id),
            None => (now, random_clock_id()),
        };

        *last_tid = Some((timestamp, clock_id));
        let tid = Tid::from_parts(timestamp, clock_id)?;

        Ok(tid)
    }
}

fn current_timestamp() -> Result<u64, SystemTimeError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros() as u64;
    Ok(timestamp)
}

fn random_clock_id() -> u16 {
    let mut rng: ChaCha20Rng = rand::make_rng();
    (rng.next_u64() & CLOCK_ID_MASK) as u16
}
