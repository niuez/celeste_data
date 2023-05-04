use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Time(u64);

impl std::ops::Add for Time {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Time(self.0 + rhs.0)
    }
}
impl std::ops::AddAssign for Time {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}
impl std::cmp::PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl std::cmp::Eq for Time {}
impl std::cmp::PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl std::cmp::Ord for Time {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
