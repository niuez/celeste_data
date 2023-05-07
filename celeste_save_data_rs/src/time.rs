use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Time(pub u64);


fn div_rem(a: u64, b: u64) -> (u64, u64) {
    (a / b, a % b)
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let v = self.0 / 10000u64;
        let (v, ms) = div_rem(v, 1000);
        let (v, s) = div_rem(v, 60);
        let (h, m) = div_rem(v, 60);
        write!(f, "{}:{:02}:{:02}.{:03}", h, m, s, ms)?;
        Ok(())
    }
}

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
impl std::ops::Sub for Time {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Time(self.0 - rhs.0)
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
