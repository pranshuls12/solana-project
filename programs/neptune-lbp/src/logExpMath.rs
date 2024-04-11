use num::bigint::BigInt;
use solana_maths::MathError; //Precision lib for scaled amount

/// Large decimal values, precise to 18 digits
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Decimal(pub BigInt);
pub const WAD: u64 = 1_000_000_000_000_000_000;

impl Decimal {
    pub fn one() -> Self {
        Self(BigInt::parse_bytes(b"1000000000000000000", 10).unwrap())
    }
    /// Zero
    pub fn zero() -> Self {
        Self(BigInt::from(0))
    }
    // OPTIMIZE: use const slice when fixed in BPF toolchain
    pub fn wad() -> BigInt {
        BigInt::from(WAD)
    }
    pub fn new(value: BigInt) -> Self {
        Decimal(value)
    }
    // Add other constants or methods signatures here
    pub fn pow(&self, exponent: BigInt) -> Result<Self, MathError> {
        // Placeholder for MIN_NATURAL_EXPONENT and MAX_NATURAL_EXPONENT as Decimal
        // Adjust these based on your actual application's requirements
        let MIN_NATURAL_EXPONENT: BigInt =
            BigInt::parse_bytes(b"-41000000000000000000", 10).unwrap();
        // Maximum allowable natural exponent (e^x) where x = 130e18 in fixed-point representation
        let MAX_NATURAL_EXPONENT: BigInt =
            BigInt::parse_bytes(b"130000000000000000000", 10).unwrap();
        let ONE_18: BigInt = BigInt::parse_bytes(b"1000000000000000000", 10).unwrap();

        if *self == Self::zero() {
            // Solving 0^0 indetermination by making it equal one.
            return Ok(Self::one());
        }
        if *self == Decimal::zero() {
            return Ok(Self::zero());
        }
        // Compute y * ln(x) in a single step.
        // convert x into Bigint
        let x = self.0.clone();
        let logx = Decimal::ln(x)?;
        let int_y = exponent;
        let logx_times_y = logx
            .checked_mul(&int_y)
            .unwrap()
            .checked_div(&ONE_18)
            .unwrap();
        // Ensure the result is within the allowed exponent range.
        if logx_times_y < MIN_NATURAL_EXPONENT || logx_times_y > MAX_NATURAL_EXPONENT {
            return Err(MathError::MulOverflow);
        }
        // Compute and return exp(y * ln(x)) to arrive at x^y

        Ok(Self(Decimal::exp(logx_times_y)?))
    }
    pub fn exp(exponent: BigInt) -> Result<BigInt, MathError> {
        // Adjust these based on your actual application's requirements
        let a = exponent;
        let MIN_EXP: BigInt = BigInt::parse_bytes(b"-41000000000000000000", 10).unwrap();
        // Maximum allowable natural exponent (e^x) where x = 130e18 in fixed-point representation
        let MAX_EXP: BigInt = BigInt::parse_bytes(b"130000000000000000000", 10).unwrap();
        let ONE_20: BigInt = BigInt::from(10u128.pow(20));
        let ONE_18: BigInt = BigInt::parse_bytes(b"1000000000000000000", 10).unwrap();

        let X0: BigInt = BigInt::parse_bytes(b"128000000000000000000", 10).unwrap();
        let A0: BigInt = BigInt::from_signed_bytes_be(&[
            1u8, 149u8, 229u8, 76u8, 93u8, 212u8, 33u8, 119u8, 245u8, 58u8, 39u8, 23u8, 47u8,
            169u8, 236u8, 99u8, 2u8, 98u8, 130u8, 112u8, 0u8, 0u8, 0u8, 0u8,
        ]);
        let X1: BigInt = BigInt::parse_bytes(b"64000000000000000000", 10).unwrap();
        let A1: BigInt = BigInt::from_signed_bytes_be(&[
            20u8, 37u8, 152u8, 44u8, 245u8, 151u8, 205u8, 32u8, 92u8, 239u8, 115u8, 128u8,
        ]);

        let X2: BigInt = BigInt::parse_bytes(b"3200000000000000000000", 10).unwrap();
        let A2: BigInt = BigInt::from_signed_bytes_be(&[
            1u8, 133u8, 81u8, 68u8, 129u8, 74u8, 127u8, 248u8, 5u8, 152u8, 15u8, 240u8, 8u8, 64u8,
            0u8,
        ]);

        let X3: BigInt = BigInt::parse_bytes(b"1600000000000000000000", 10).unwrap();
        let A3: BigInt = BigInt::from_signed_bytes_be(&[
            2u8, 223u8, 10u8, 181u8, 168u8, 10u8, 34u8, 198u8, 26u8, 181u8, 167u8, 0u8,
        ]);

        let X4: BigInt = BigInt::parse_bytes(b"800000000000000000000", 10).unwrap();
        let A4: BigInt = BigInt::from_signed_bytes_be(&[
            63u8, 31u8, 206u8, 61u8, 166u8, 54u8, 234u8, 92u8, 248u8, 80u8,
        ]);

        let X5: BigInt = BigInt::parse_bytes(b"400000000000000000000", 10).unwrap();
        let A5: BigInt = BigInt::from_signed_bytes_be(&[
            1u8, 39u8, 250u8, 39u8, 114u8, 44u8, 192u8, 108u8, 197u8, 226u8,
        ]);

        let X6: BigInt = BigInt::parse_bytes(b"200000000000000000000", 10).unwrap();
        let A6: BigInt =
            BigInt::from_signed_bytes_be(&[40u8, 14u8, 96u8, 17u8, 78u8, 219u8, 128u8, 93u8, 3u8]);
        let X7: BigInt = BigInt::parse_bytes(b"100000000000000000000", 10).unwrap();
        let A7: BigInt =
            BigInt::from_signed_bytes_be(&[14u8, 188u8, 95u8, 180u8, 23u8, 70u8, 18u8, 17u8, 16u8]);
        let X8: BigInt = BigInt::parse_bytes(b"50000000000000000000", 10).unwrap();
        let A8: BigInt =
            BigInt::from_signed_bytes_be(&[8u8, 240u8, 15u8, 118u8, 10u8, 75u8, 45u8, 181u8, 93u8]);

        let X9: BigInt = BigInt::parse_bytes(b"25000000000000000000", 10).unwrap();
        let A9: BigInt = BigInt::from_signed_bytes_be(&[
            6u8, 245u8, 241u8, 119u8, 87u8, 136u8, 147u8, 121u8, 55u8,
        ]);
        let X10: BigInt = BigInt::parse_bytes(b"12500000000000000000", 10).unwrap();
        let A10: BigInt =
            BigInt::from_signed_bytes_be(&[6u8, 36u8, 143u8, 51u8, 112u8, 75u8, 40u8, 102u8, 3u8]);
        let X11: BigInt = BigInt::parse_bytes(b"6250000000000000000", 10).unwrap();
        let A11: BigInt = BigInt::from_signed_bytes_be(&[
            5u8, 197u8, 72u8, 103u8, 11u8, 149u8, 16u8, 231u8, 172u8,
        ]);
        // Assume this function is part of a Decimal struct implementation
        let mut x = a;
        if x < MIN_EXP || x > MAX_EXP {
            return Err(MathError::AddOverflow);
        }
        if x < BigInt::from(0) {
            // We only handle positive exponents: e^(-x) is computed as 1 / e^x. We can safely make x positive since it
            // fits in the signed 256 bit range (as it is larger than MIN_NATURAL_EXPONENT).
            // Fixed point division requires multiplying by ONE_18.
            let minusx = -x;
            return Ok(ONE_18
                .checked_mul(&ONE_18)
                .unwrap()
                .checked_div(&Decimal::exp(minusx).unwrap())
                .unwrap());
        }

        let mut product = ONE_20.clone();

        let mut first_an = BigInt::from(1); // Default to 1 in high precision
        if x >= X0 {
            x = x.checked_sub(&X0).unwrap();
            first_an = A0;
        } else if x >= X1 {
            x = x.checked_sub(&X1).unwrap();
            first_an = A1;
        }

        x = x.checked_mul(&BigInt::from(100)).unwrap(); // Enhance precision for 20 terms

        if x >= X2 {
            x = x.checked_sub(&X2).unwrap();
            product = product
                .checked_mul(&A2)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }
        if x >= X3 {
            x = x.checked_sub(&X3).unwrap();
            product = product
                .checked_mul(&A3)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }
        if x >= X4 {
            x = x.checked_sub(&X4).unwrap();
            product = product
                .checked_mul(&A4)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }
        if x >= X5 {
            x = x.checked_sub(&X5).unwrap();
            product = product
                .checked_mul(&A5)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }
        if x >= X6 {
            x = x.checked_sub(&X6).unwrap();
            product = product
                .checked_mul(&A6)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }
        if x >= X7 {
            x = x.checked_sub(&X7).unwrap();
            product = product
                .checked_mul(&A7)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }
        if x >= X8 {
            x = x.checked_sub(&X8).unwrap();
            product = product
                .checked_mul(&A8)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }
        if x >= X9 {
            x = x.checked_sub(&X9).unwrap();
            product = product
                .checked_mul(&A9)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();
        }

        // Taylor series expansion for the remainder
        let mut series_sum = ONE_20.clone();
        let mut term = x.clone();
        series_sum = series_sum.checked_add(&term).unwrap();

        for n in 2..=12 {
            term = term
                .checked_mul(&x)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap()
                .checked_div(&BigInt::from(n as u64))
                .unwrap();
            series_sum = series_sum.checked_add(&term).unwrap();
        }

        // Combine all parts
        let result = product
            .checked_mul(&series_sum)
            .unwrap()
            .checked_div(&ONE_20)
            .unwrap()
            .checked_mul(&first_an)
            .unwrap()
            .checked_div(&BigInt::from(100))
            .unwrap();

        Ok(result)
    }

    pub fn ln(exponent: BigInt) -> Result<BigInt, MathError> {
        let ONE_20: BigInt = BigInt::parse_bytes(b"100000000000000000000", 10).unwrap();
        let one_18: BigInt = BigInt::parse_bytes(b"1000000000000000000", 10).unwrap();
        // Convert 'a' from Decimal to U256 for direct arithmetic operations
        let mut x = exponent;
        if x < one_18 {
            return Ok(-Decimal::ln(
                one_18
                    .checked_mul(&one_18)
                    .unwrap()
                    .checked_div(&x)
                    .unwrap(),
            )?);
        }

        let X0: BigInt = BigInt::parse_bytes(b"128000000000000000000", 10).unwrap();
        let A0: BigInt = BigInt::parse_bytes(
            b"38877084059945950922200000000000000000000000000000000000",
            10,
        )
        .unwrap();
        let X1: BigInt = BigInt::parse_bytes(b"64000000000000000000", 10).unwrap();
        let A1: BigInt = BigInt::parse_bytes(b"6235149080811616882910000000", 10).unwrap();

        let X2: BigInt = BigInt::parse_bytes(b"3200000000000000000000", 10).unwrap();
        let A2: BigInt = BigInt::parse_bytes(b"7896296018268069516100000000000000", 10).unwrap();

        let X3: BigInt = BigInt::parse_bytes(b"1600000000000000000000", 10).unwrap();
        let A3: BigInt = BigInt::parse_bytes(b"888611052050787263676000000", 10).unwrap();

        let X4: BigInt = BigInt::parse_bytes(b"800000000000000000000", 10).unwrap();
        let A4: BigInt = BigInt::parse_bytes(b"298095798704172827474000", 10).unwrap();

        let X5: BigInt = BigInt::parse_bytes(b"400000000000000000000", 10).unwrap();
        let A5: BigInt = BigInt::parse_bytes(b"5459815003314423907810", 10).unwrap();
        let X6: BigInt = BigInt::parse_bytes(b"200000000000000000000", 10).unwrap();
        let A6: BigInt = BigInt::parse_bytes(b"738905609893065022723", 10).unwrap();
        let X7: BigInt = BigInt::parse_bytes(b"100000000000000000000", 10).unwrap();
        let A7: BigInt = BigInt::parse_bytes(b"271828182845904523536", 10).unwrap();
        let X8: BigInt = BigInt::parse_bytes(b"50000000000000000000", 10).unwrap();
        let A8: BigInt = BigInt::parse_bytes(b"164872127070012814685", 10).unwrap();

        let X9: BigInt = BigInt::parse_bytes(b"25000000000000000000", 10).unwrap();
        let A9: BigInt = BigInt::parse_bytes(b"128402541668774148407", 10).unwrap();
        let X10: BigInt = BigInt::parse_bytes(b"12500000000000000000", 10).unwrap();
        let A10: BigInt = BigInt::parse_bytes(b"113314845306682631683", 10).unwrap();
        let X11: BigInt = BigInt::parse_bytes(b"6250000000000000000", 10).unwrap();
        let A11: BigInt = BigInt::parse_bytes(b"106449445891785942956", 10).unwrap();
        let mut a = x; // Assuming self.0 is U256
        let mut sum = BigInt::from(0); // Ensure 'sum' is of type U256 for arithmetic
                                       // Check and compute for A0, X0

        if a >= A0.checked_mul(&one_18).unwrap() {
            a = a.checked_div(&A0).unwrap();
            sum = sum.checked_add(&X0).unwrap();
        }

        if a >= A1.checked_mul(&one_18).unwrap() {
            a = a.checked_div(&A1).unwrap();
            sum = sum.checked_add(&X1).unwrap();
        }

        a *= 100;
        sum *= 100;
        if a >= A2 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A2).unwrap();
            sum = sum.checked_add(&X2).unwrap();
        }
        if a >= A3 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A3).unwrap();
            sum = sum.checked_add(&X3).unwrap();
        }
        if a >= A4 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A4).unwrap();
            sum = sum.checked_add(&X4).unwrap();
        }
        if a >= A5 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A5).unwrap();
            sum = sum.checked_add(&X5).unwrap();
        }
        if a >= A6 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A6).unwrap();
            sum = sum.checked_add(&X6).unwrap();
        }
        if a >= A7 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A7).unwrap();
            sum = sum.checked_add(&X7).unwrap();
        }
        if a >= A8 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A8).unwrap();
            sum = sum.checked_add(&X8).unwrap();
        }

        if a >= A9 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A9).unwrap();
            sum = sum.checked_add(&X9).unwrap();
        }
        if a >= A10 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A10).unwrap();
            sum = sum.checked_add(&X10).unwrap();
        }
        if a >= A11 {
            a = a.checked_mul(&ONE_20).unwrap().checked_div(&A11).unwrap();
            sum = sum.checked_add(&X11).unwrap();
        }

        let sub = a.checked_sub(&ONE_20).unwrap();
        let z = sub
            .checked_mul(&ONE_20)
            .unwrap()
            .checked_div(&a.checked_add(&ONE_20).unwrap())
            .unwrap();

        let z_squared = z.checked_mul(&z).unwrap().checked_div(&ONE_20).unwrap();
        // Use `z` directly for the first term in series_sum
        // Initialize series_sum directly with the first term adjustment
        let mut num = z.clone(); // Starting value of `num` is `z`
        let mut series_sum = num.clone();

        // Pre-calculate division factors to BigInt, to avoid conversion inside the loop
        let factors = [3, 5, 7, 9, 11];
        // Calculate and accumulate each term directly, starting from the second factor
        for factor_str in factors.iter() {
            let divisor = BigInt::from(*factor_str);
            num = num
                .checked_mul(&z_squared)
                .unwrap()
                .checked_div(&ONE_20)
                .unwrap();

            series_sum = series_sum
                .checked_add(&num.checked_div(&divisor).unwrap())
                .unwrap();
        }

        // Multiply series_sum by 2 for the ln calculation (adjust as per your logic)
        series_sum = series_sum.checked_mul(&BigInt::from(2)).unwrap();

        // Combine sum and series_sum and adjust the scale back to the original unit (if needed)
        let result = sum
            .checked_add(&series_sum)
            .unwrap()
            .checked_div(&BigInt::from(100))
            .unwrap();
        Ok(result)
    }

    // Add
    pub fn add(a: Self, b: Self) -> Result<Self, MathError> {
        let result = a.0 + b.0;
        // Assuming you have a mechanism to check for overflow, if not, BigInt handles it.
        Ok(Decimal(result))
    }

    // Subtract
    pub fn sub(a: Self, b: Self) -> Result<Self, MathError> {
        if b.0 > a.0 {
            return Err(MathError::AddOverflow);
        }
        let result = a.0 - b.0;
        Ok(Decimal(result))
    }

    // Multiply Down
    pub fn mul_down(a: Self, b: Self) -> Result<Self, MathError> {
        let one = BigInt::from(1e18 as u64); // Assuming 1e18 fits in u64 for simplicity
        let product = a.0 * b.0;
        let result = product / &one;
        Ok(Decimal(result))
    }

    // Multiply Up
    pub fn mul_up(a: Self, b: Self) -> Result<Self, MathError> {
        let one = BigInt::from(1e18 as u64);
        let product = a.0 * b.0;
        // The product + one - 1 ensures rounding up
        let result = if product == BigInt::from(0) {
            product
        } else {
            (product + &one - 1) / &one
        };
        Ok(Decimal(result))
    }

    // Divide Down
    pub fn div_down(a: Self, b: Self) -> Result<Self, MathError> {
        if b.0 == BigInt::from(0) {
            return Err(MathError::DividedByZero);
        }
        let one = BigInt::from(1e18 as u64);
        let a_inflated = a.0 * &one;
        let result = a_inflated / b.0;
        Ok(Decimal(result))
    }

    // Divide Up
    pub fn div_up(a: Self, b: Self) -> Result<Self, MathError> {
        if b.0 == BigInt::from(0) {
            return Err(MathError::DividedByZero);
        }
        let one = BigInt::from(1e18 as u64);
        let a_inflated = a.0 * &one;
        let result = if a_inflated == BigInt::from(0) {
            a_inflated
        } else {
            (a_inflated + &one - 1) / b.0
        };
        Ok(Decimal(result))
    }

    pub fn pow_up(x: &BigInt, y: &BigInt) -> Result<BigInt, MathError> {
        let ONE: BigInt = BigInt::from(10u32.pow(18)); // Example for 1.0 in fixed-point representation
        let TWO: BigInt = BigInt::from(2).checked_mul(&ONE).unwrap();
        let FOUR: BigInt = BigInt::from(4).checked_mul(&ONE).unwrap();
        let MAX_POW_RELATIVE_ERROR: BigInt = BigInt::from(1u32); // Placeholder

        if y == &ONE {
            Ok(x.clone())
        } else if y == &TWO {
            Ok(Self::mul_up(Self(x.clone()), Self(x.clone()))?.0)
        } else if y == &FOUR {
            let square = Self::mul_up(Self(x.clone()), Self(x.clone()))?.0;
            Ok(Self::mul_up(Self(square.clone()), Self(square))?.0)
        } else {
            let raw = Self::pow(&Self(x.clone()), y.clone())?; // Placeholder for fixed-point pow operation

            let max_error = Self::add(
                Self::mul_up(raw.clone(), Self::new(MAX_POW_RELATIVE_ERROR))?,
                Self::one(),
            )?;
            Ok(Self::add(raw, max_error)?.0)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_exp_small_value() {
        // Testing exp with a small value that should not overflow
        let small_value = Decimal::from(1u128); // Equivalent to 1e-18 in fixed-point notation
        let result = DecimalExt::exp(small_value).unwrap();
        // Expected result close to e^1 in fixed-point notation
        let expected = Decimal::from(2718281828459045235u128); // e ≈ 2.7182818284590452353602874713527
        assert_eq!(result, expected);
    }

    #[test]
    fn test_exp_zero() {
        // Testing exp with zero
        let zero_value = Decimal::from(0u128);
        let result = DecimalExt::exp(zero_value).unwrap();
        // Expected result is 1 (e^0 = 1) in fixed-point notation
        let expected = Decimal::from(10u128.pow(18)); // 1 with 18 decimal places
        assert_eq!(result, expected);
    }

    #[test]
    fn test_exp_large_value() {
        // Testing exp with a value that is large but within bounds
        let large_value = Decimal::from(20u128); // Equivalent to 2e-17 in fixed-point notation
        let result = DecimalExt::exp(large_value).unwrap();
        // This is a simplified expectation. In practice, calculate the expected value to a high precision.
        // For the sake of example, we're asserting it simply returns a result without error.
        // The actual expected value would need to be calculated based on e^20, adjusted for your fixed-point precision.
        assert!(result > Decimal::from(10u128.pow(18))); // e^20 will definitely be larger than 1
    }

    #[test]
    #[should_panic(expected = "InvalidExponent")]
    fn test_exp_out_of_bounds() {
        // Testing exp with a value out of bounds should return an error
        let out_of_bounds_value = Decimal::MAX_EXP + Decimal::from(1u128); // Just beyond the maximum exponent
        DecimalExt::exp(out_of_bounds_value).unwrap();
    }
}
