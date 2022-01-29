use std::time::Duration;
use crate::time::ArchivedDuration;

impl PartialEq<Duration> for ArchivedDuration {
    #[inline]
    fn eq(&self, other: &Duration) -> bool {
        return (self.as_nanos() == other.as_nanos()) && (self.as_secs() == other.as_secs());
    }
}

impl PartialEq<ArchivedDuration> for Duration {
    #[inline]
    fn eq(&self, other: &ArchivedDuration) -> bool {
        return other.eq(self)
    }
}