use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops;

/// Specifies a fraction type which is used to keep measure the positions and durations of notes.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct Fraction {
    numerator: i32,
    denominator: i32,
}

impl Fraction {
    /// Create a fraction from a numerator and denominator.
    pub fn new(numerator: i32, denominator: i32) -> Fraction {
        let (numerator, denominator) = Fraction::balance(numerator, denominator);
        Fraction {
            numerator,
            denominator,
        }
    }

    /// Create a zero fraction.
    pub fn zero() -> Fraction {
        Fraction {
            numerator: 0,
            denominator: 1,
        }
    }

    /// Check if the fraction is zero.
    pub fn is_zero(&self) -> bool {
        self.numerator == 0
    }

    /// Find the greatest common divisor of two numbers.
    fn gcd(a: i32, b: i32) -> i32 {
        if a == 0 {
            return b;
        }
        Fraction::gcd(b % a, a)
    }

    /// Balance a fraction.
    pub fn balance(numerator: i32, denominator: i32) -> (i32, i32) {
        // Make sure negative sign is on top
        let numerator = if denominator < 0 {
            -numerator
        } else {
            numerator
        };
        let denominator = denominator.abs();
        let gcd = Fraction::gcd(numerator, denominator).abs();
        (numerator / gcd, denominator / gcd)
    }

    /// Truncate the fraction to a whole number.
    pub fn to_whole(&self) -> i32 {
        self.numerator / self.denominator
    }

    /// Get the numerator.
    pub fn numerator(&self) -> i32 {
        self.numerator
    }

    /// Get the denominator.
    pub fn denominator(&self) -> i32 {
        self.denominator
    }
}

impl ops::Add<Fraction> for Fraction {
    type Output = Fraction;

    /// Add two fractions together.
    fn add(self, rhs: Fraction) -> Fraction {
        if self.denominator == rhs.denominator {
            return Fraction::new(self.numerator + rhs.numerator, self.denominator);
        }
        let lhs_numerator = self.numerator * rhs.denominator;
        let rhs_numerator = rhs.numerator * self.denominator;
        let denominator = self.denominator * rhs.denominator;
        Fraction::new(lhs_numerator + rhs_numerator, denominator)
    }
}

impl ops::Sub<Fraction> for Fraction {
    type Output = Fraction;

    /// Subtract one fraction from the other.
    fn sub(self, rhs: Fraction) -> Fraction {
        if self.denominator == rhs.denominator {
            return Fraction::new(self.numerator - rhs.numerator, self.denominator);
        }
        let lhs_numerator = self.numerator * rhs.denominator;
        let rhs_numerator = rhs.numerator * self.denominator;
        let denominator = self.denominator * rhs.denominator;
        Fraction::new(lhs_numerator - rhs_numerator, denominator)
    }
}

impl ops::Mul<Fraction> for Fraction {
    type Output = Fraction;

    /// Multiply two fractions together.
    fn mul(self, rhs: Fraction) -> Fraction {
        Fraction::new(
            self.numerator * rhs.numerator,
            self.denominator * rhs.denominator,
        )
    }
}

impl ops::Div<Fraction> for Fraction {
    type Output = Fraction;

    /// Divide one fraction my the other.
    fn div(self, rhs: Fraction) -> Fraction {
        Fraction::new(
            self.numerator * rhs.denominator,
            self.denominator * rhs.numerator,
        )
    }
}

impl ops::AddAssign<Fraction> for Fraction {
    /// Add a fraction in place.
    fn add_assign(&mut self, rhs: Fraction) {
        let lhs_numerator = self.numerator * rhs.denominator;
        let rhs_numerator = rhs.numerator * self.denominator;
        let denominator = self.denominator * rhs.denominator;
        let numerator = lhs_numerator + rhs_numerator;
        let (numerator, denominator) = Fraction::balance(numerator, denominator);
        self.numerator = numerator;
        self.denominator = denominator;
    }
}

impl ops::SubAssign<Fraction> for Fraction {
    /// Subtract a fraction in place.
    fn sub_assign(&mut self, rhs: Fraction) {
        let lhs_numerator = self.numerator * rhs.denominator;
        let rhs_numerator = rhs.numerator * self.denominator;
        let denominator = self.denominator * rhs.denominator;
        let numerator = lhs_numerator - rhs_numerator;
        let (numerator, denominator) = Fraction::balance(numerator, denominator);
        self.numerator = numerator;
        self.denominator = denominator;
    }
}

impl ops::MulAssign<Fraction> for Fraction {
    /// Multiply a fraction in place.
    fn mul_assign(&mut self, rhs: Fraction) {
        let numerator = self.numerator * rhs.numerator;
        let denominator = self.denominator * rhs.denominator;
        let (numerator, denominator) = Fraction::balance(numerator, denominator);
        self.numerator = numerator;
        self.denominator = denominator;
    }
}

impl ops::DivAssign<Fraction> for Fraction {
    /// Divide by a fraction in place.
    fn div_assign(&mut self, rhs: Fraction) {
        let numerator = self.numerator * rhs.denominator;
        let denominator = self.denominator * rhs.numerator;
        let (numerator, denominator) = Fraction::balance(numerator, denominator);
        self.numerator = numerator;
        self.denominator = denominator;
    }
}

impl ops::Neg for Fraction {
    type Output = Fraction;

    /// Negate a fraction.
    fn neg(self) -> Fraction {
        Fraction::new(-self.numerator, self.denominator)
    }
}

impl ops::Rem for Fraction {
    type Output = Fraction;

    /// Get the remainder of a fraction.
    fn rem(self, rhs: Self) -> Fraction {
        self - Fraction::new((self / rhs).to_whole(), 1) * rhs
    }
}

impl std::cmp::PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Fraction) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Fraction {
    /// Compare two fractions.
    fn cmp(&self, other: &Fraction) -> Ordering {
        let self_num = self.numerator * other.denominator;
        let other_num = other.numerator * self.denominator;
        self_num.cmp(&other_num)
    }
}

impl Default for Fraction {
    /// The default fraction is zero.
    fn default() -> Self {
        Self::zero()
    }
}


#[cfg(test)]
mod tests {
    use crate::fraction::Fraction;

    #[test]
    fn fraction_balance() {
        let fraction = Fraction::new(6, 3);
        assert_eq!(Fraction::new(2, 1), fraction);

        let fraction = Fraction::new(2, 4);
        assert_eq!(Fraction::new(1, 2), fraction);
    }

    #[test]
    fn fraction_add() {
        let fraction1 = Fraction::new(1, 1);
        let fraction2 = Fraction::new(2, 1);
        assert_eq!(Fraction::new(3, 1), fraction1 + fraction2);

        let fraction1 = Fraction::new(1, 2);
        let fraction2 = Fraction::new(1, 2);
        assert_eq!(Fraction::new(1, 1), fraction1 + fraction2);

        let fraction1 = Fraction::new(1, 2);
        let fraction2 = Fraction::new(2, 3);
        assert_eq!(Fraction::new(7, 6), fraction1 + fraction2);
    }

    #[test]
    fn fraction_subtract() {
        let fraction1 = Fraction::new(2, 1);
        let fraction2 = Fraction::new(1, 1);
        assert_eq!(Fraction::new(1, 1), fraction1 - fraction2);

        let fraction1 = Fraction::new(1, 2);
        let fraction2 = Fraction::new(1, 2);
        assert_eq!(Fraction::zero(), fraction1 - fraction2);

        let fraction1 = Fraction::new(2, 3);
        let fraction2 = Fraction::new(1, 2);
        assert_eq!(Fraction::new(1, 6), fraction1 - fraction2);
    }

    #[test]
    fn fraction_multiply() {
        let fraction1 = Fraction::new(1, 1);
        let fraction2 = Fraction::new(2, 1);
        assert_eq!(Fraction::new(2, 1), fraction1 * fraction2);

        let fraction1 = Fraction::new(1, 2);
        let fraction2 = Fraction::new(1, 2);
        assert_eq!(Fraction::new(1, 4), fraction1 * fraction2);

        let fraction1 = Fraction::new(1, 2);
        let fraction2 = Fraction::new(2, 3);
        assert_eq!(Fraction::new(1, 3), fraction1 * fraction2);
    }

    #[test]
    fn fraction_divide() {
        let fraction1 = Fraction::new(1, 1);
        let fraction2 = Fraction::new(2, 1);
        assert_eq!(Fraction::new(1, 2), fraction1 / fraction2);

        let fraction1 = Fraction::new(1, 2);
        let fraction2 = Fraction::new(1, 2);
        assert_eq!(Fraction::new(1, 1), fraction1 / fraction2);

        let fraction1 = Fraction::new(1, 2);
        let fraction2 = Fraction::new(2, 3);
        assert_eq!(Fraction::new(3, 4), fraction1 / fraction2);
    }
}
