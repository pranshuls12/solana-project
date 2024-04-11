use crate::logExpMath::Decimal;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use num::bigint::BigInt;
use solana_maths::MathError; //Precision lib for scaled amount
use solana_safe_math::SafeMath;
use std::result::Result;
use uint::construct_uint;

// U256 with 192 bits consisting of 3 x 64-bit words
construct_uint! {
    pub struct U256(4);
}
/*/// Large decimal values, precise to 18 digits
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Decimal(pub U256);
*/
// Stateless implementation
pub struct WeightedMath;

pub trait WeightedMathTrait {
    fn get_scaling_factor(decimals: u8) -> u128;
    fn normalize_weight(percentage: u8) -> BigInt;
    fn scale_value(value: u64, scaling_factor: u128) -> BigInt;
    fn downscale_value(scaled_value: &BigInt, scaling_factor: u128) -> Result<u64, ProgramError>;
    fn calculate_invariant(
        normalized_weights: &[BigInt],
        balances: &[BigInt],
    ) -> Result<BigInt, ProgramError>;
    fn calculate_weight_change_progress(
        current_time: i64,
        start_time: i64,
        end_time: i64,
    ) -> Result<u64, ProgramError>;
    fn interpolate_weight(
        start_weight: u8,
        end_weight: u8,
        percent_progress: u64,
    ) -> Result<u8, ProgramError>;
    fn calc_bpt_out_given_exact_tokens_in(
        balances: &[BigInt],
        normalized_weights: &[BigInt],
        amounts_in: &[BigInt],
        bpt_total_supply: &BigInt,
        swap_fee_percentage: &BigInt,
    ) -> Result<u64, ProgramError>;
    fn calc_out_given_in(
        balance_in: &BigInt,
        weight_in: &BigInt,
        balance_out: &BigInt,
        weight_out: &BigInt,
        amount_in: &BigInt,
    ) -> Result<BigInt, MathError>;
    fn calc_in_given_out(
        balance_in: &BigInt,
        weight_in: &BigInt,
        balance_out: &BigInt,
        weight_out: &BigInt,
        amount_out: &BigInt,
    ) -> Result<BigInt, MathError>;

    fn compute_proportional_amounts_out(
        balances: &[BigInt],
        bpt_total_supply: &BigInt,
        bpt_amount_in: &BigInt,
    ) -> Result<Vec<BigInt>, MathError>;
    fn calc_due_fee_amount(amount: u64, fee_percentage: u8) -> u64;
    /*  fn calc_token_in_given_exact_bpt_out(balance: u128, normalized_weight: u128, bpt_amount_out: u128, bpt_total_supply: u128, swap_fee_percentage: u128) -> Result<U128, MathError>;
    fn calc_bpt_in_given_exact_tokens_out(balances: &[u128], normalized_weights: &[u128], amounts_out: &[u128], bpt_total_supply: u128, swap_fee_percentage: u128) -> Result<U128, MathError>;
    fn calc_token_out_given_exact_bpt_in(balance: u128, normalized_weight: u128, bpt_amount_in: u128, bpt_total_supply: u128, swap_fee_percentage: u128) -> Result<U128, MathError>;
    fn calc_tokens_out_given_exact_bpt_in(balances: &[u128], bpt_amount_in: u128, total_bpt: u128) -> Result<Vec<u128>, MathError>;
    fn calc_due_token_protocol_swap_fee_amount(balance: u128, normalized_weight: u128, previous_invariant: u128, current_invariant: u128, protocol_swap_fee_percentage: u128) -> Result<U128, MathError>;
    */
}

impl WeightedMathTrait for WeightedMath {
    // Return range between [0-1] scaled version of percentage
    fn normalize_weight(percentage: u8) -> BigInt {
        let normalized_weight = BigInt::from(percentage as u128 * 10u128.pow(16));
        normalized_weight
    }
    // Function to calculate the scaling factor based on the token's decimals.
    fn get_scaling_factor(decimals: u8) -> u128 {
        let scaling_factor = 10u128.pow((18 - decimals as u32).try_into().unwrap_or_default());

        scaling_factor
    }
    // Return (10^18) scaled version of percentage
    fn scale_value(value: u64, scaling_factor: u128) -> BigInt {
        let value_as_u128 = u128::from(value);

        // Perform the multiplication to get the scaled value.
        let scaled_value = value_as_u128 * scaling_factor;

        // Use the 'from_scaled_val' method of the Decimal struct to create and return a Decimal instance
        // from the scaled value.
        BigInt::from(scaled_value)
    }
    fn downscale_value(scaled_value: &BigInt, scaling_factor: u128) -> Result<u64, ProgramError> {
        // The scaling_factor is multiplied by 10^9 in the scale_value function,
        // so we need to divide by scaling_factor * 10^9 to reverse the scaling.
        let divisor = BigInt::from(scaling_factor);
        //msg!("downscaling {:?} by {:?}", scaled_value, divisor);
        // Perform the division to get the original value.
        let original_value_bigint = scaled_value.checked_div(&divisor).unwrap();

        // Attempt to convert the BigInt back to u64.
        // This can fail if the original value was too large or if precision was lost.
        Ok(original_value_bigint.to_u64_digits().1[0])
    }

    fn calculate_invariant(
        normalized_weights: &[BigInt],
        balances: &[BigInt],
    ) -> Result<BigInt, ProgramError> {
        if normalized_weights.len() != balances.len() {
            return Err(MathError::MulOverflow.into()); // Adjust to a more specific error if needed
        }
        let one = BigInt::from(10u128.pow(18));

        let mut invariant = BigInt::from(1);
        for i in 0..normalized_weights.len() {
            let power_balance = Decimal::new(balances[i].clone())
                .pow(normalized_weights[i].clone())
                .unwrap();
            invariant = invariant.checked_mul(&power_balance.0).unwrap();
        }
        // now the calculation  is over we convert back to u64
        invariant = invariant.checked_div(&one).unwrap();

        //msg!("invariant {:?}", invariant);

        Ok(invariant)
    }
    fn calculate_weight_change_progress(
        current_time: i64,
        start_time: i64,
        end_time: i64,
    ) -> Result<u64, ProgramError> {
        if current_time >= end_time {
            return Ok(1u64);
        } else if current_time < start_time {
            return Ok(0u64);
        }
        /*msg!(
            "Calculating pct cu{:?},st{:?}-end{:?}",
            current_time,
            start_time,
            end_time
        );*/

        let total_seconds: i64 = end_time - start_time;
        let second_elapsed: i64 = current_time - start_time;
        //safe since total second can't be 0
        let delta = second_elapsed * 100 / total_seconds;
        //msg!("delta {:?}", delta);

        let mut bytes = delta.to_le_bytes();
        let delta_64 = u64::from_le_bytes(bytes);

        Ok(delta_64)
    }
    // returns the current applicable weight
    // We might need more precision here
    fn interpolate_weight(
        start_weight_u8: u8,
        end_weight_u8: u8,
        percent_progress: u64,
    ) -> Result<u8, ProgramError> {
        let start_weight: u64 = start_weight_u8.try_into().unwrap();
        let end_weight: u64 = end_weight_u8.try_into().unwrap();
        //msg!("interpolating {:?},{:?}", start_weight, end_weight);

        let mut interpolation: u64;
        if percent_progress == 0u64 || start_weight == end_weight {
            interpolation = start_weight;
        }
        if percent_progress >= 100u64 {
            interpolation = end_weight;
        }

        if start_weight > end_weight {
            let weight_delta = percent_progress * (start_weight - end_weight);
            interpolation = (start_weight * 100 - weight_delta) / 100;
        } else {
            let weight_delta = percent_progress * (end_weight - start_weight);
            interpolation = (start_weight * 100 + weight_delta) / 100;
        }
        /*msg!(
            "start-weight {:?}, endweight {:?}, interpolataion cu{:?}",
            start_weight,
            end_weight,
            interpolation
        );*/
        Ok(interpolation.try_into().unwrap())
    }
    fn calc_bpt_out_given_exact_tokens_in(
        balances: &[BigInt],
        normalized_weights: &[BigInt],
        amounts_in: &[BigInt],
        bpt_total_supply: &BigInt,
        swap_fee_percentage: &BigInt,
    ) -> Result<u64, ProgramError> {
        if balances.len() != normalized_weights.len() || balances.len() != amounts_in.len() {
            return Err(MathError::MulOverflow.into());
        }

        let mut balance_ratios_with_fee = Vec::with_capacity(amounts_in.len());
        let mut invariant_ratio_with_fees = BigInt::from(0);
        let fixed_point_one = BigInt::from(1 * 10u64.pow(18)); // Assuming fixed point arithmetic with a scale factor
        for i in 0..balances.len() {
            let balance_ratio_with_fee = balances[i]
                .checked_add(&amounts_in[i])
                .unwrap()
                .checked_div(&balances[i])
                .unwrap();
            balance_ratios_with_fee.push(balance_ratio_with_fee.clone());
            invariant_ratio_with_fees = invariant_ratio_with_fees
                .checked_add(
                    &balance_ratio_with_fee
                        .checked_mul(&normalized_weights[i])
                        .unwrap(),
                )
                .unwrap();
        }

        let mut invariant_ratio = fixed_point_one.clone();
        for i in 0..balances.len() {
            let amount_in_without_fee = if balance_ratios_with_fee[i] > invariant_ratio_with_fees {
                let non_taxable_amount = balances[i]
                    .checked_mul(
                        &invariant_ratio_with_fees
                            .checked_sub(&fixed_point_one)
                            .unwrap(),
                    )
                    .unwrap();
                let taxable_amount = amounts_in[i].checked_sub(&non_taxable_amount).unwrap();
                non_taxable_amount.checked_add(
                    &taxable_amount
                        .checked_mul(&fixed_point_one.checked_sub(swap_fee_percentage).unwrap())
                        .unwrap(),
                )
            } else {
                Some(amounts_in[i].clone())
            };

            let balance_ratio = Decimal::new(
                balances[i]
                    .checked_add(&amount_in_without_fee.unwrap())
                    .unwrap()
                    .checked_div(&balances[i])
                    .unwrap(),
            );
            let weight = normalized_weights[i].clone();
            invariant_ratio = invariant_ratio
                .checked_mul(&balance_ratio.pow(weight).unwrap().0)
                .unwrap();
        }

        if invariant_ratio > fixed_point_one {
            let result = bpt_total_supply
                .checked_mul(&invariant_ratio.checked_sub(&fixed_point_one).unwrap()) // here removing extra from decimal
                .unwrap()
                .checked_div(&BigInt::from(1 * 10u64.pow(18)))
                .unwrap()
                .checked_div(&BigInt::from(1 * 10u64.pow(18)))
                .unwrap();
            /*msg!(
                "Calculataion initial supply:  {:?} result {:?}",
                bpt_total_supply,
                result
            );*/
            let buint_result = bpt_total_supply
                .checked_sub(&result)
                .unwrap()
                .checked_div(&BigInt::from(1 * 10u64.pow(18)))
                .unwrap()
                .to_biguint()
                .unwrap();
            let digits = buint_result.to_u64_digits();
            //msg!("u64 conversion {:?}", digits);
            Ok(digits[0])
        } else {
            Ok(0u64)
        }
    }
    // Calculates the due token protocol swap fee amount, using BigInt for calculations and converting to Decimal only for the power operation.
    fn calc_due_fee_amount(amount: u64, fee_percentage: u8) -> u64 {
        amount * fee_percentage as u64 / 100u64
    }

    /*
    fn get_due_protocol_fee_amounts(
        balances: Vec<BigInt>,
        normalized_weights: Vec<BigInt>,
        max_weight_token_index: usize,
        previous_invariant: &BigInt,
        current_invariant: &BigInt,
        protocol_swap_fee_percentage: &BigInt,
    ) -> Result<Vec<BigInt>, MathError> {
        let total_tokens = balances.len();
        let mut due_protocol_fee_amounts = vec![BigInt::zero(); total_tokens];

        if protocol_swap_fee_percentage.is_zero() {
            return Ok(due_protocol_fee_amounts);
        }

        let fee_amount = calc_due_token_protocol_swap_fee_amount(
            &balances[max_weight_token_index],
            &normalized_weights[max_weight_token_index],
            previous_invariant,
            current_invariant,
            protocol_swap_fee_percentage,
        )?;

        // Assuming that only the token with the max weight index gets its fee calculated and set
        due_protocol_fee_amounts[max_weight_token_index] = fee_amount;

        Ok(due_protocol_fee_amounts)
    }*/
    fn calc_out_given_in(
        balance_in: &BigInt,
        weight_in: &BigInt,
        balance_out: &BigInt,
        weight_out: &BigInt,
        amount_in: &BigInt,
    ) -> Result<BigInt, MathError> {
        /*// Ensure amountIn is less than balanceIn * MAX_IN_RATIO
        let max_in_ratio = BigInt::from(10u32); // Placeholder value, set your max ratio

        if amount_in > &(balance_in * &max_in_ratio) {
            msg!("amount in above the balance by the max ration");

            return Err(MathError::DividedByZero);
        }
        msg!("Calc out given in");*/
        let denominator = Decimal(balance_in.checked_add(&amount_in).unwrap());
        let base = Decimal::div_up(Decimal(balance_in.clone()), denominator.clone()).unwrap();
        /*msg!(
            "Calc out given in: balance_in {:?}, denominator {:?}",
            base,
            denominator
        );*/

        let exponent =
            Decimal::div_down(Decimal(weight_in.clone()), Decimal(weight_out.clone())).unwrap();
        /*msg!(
            "Calc out given in: base {:?}, denominator {:?}, exponent {:?}",
            base,
            denominator,
            exponent
        );*/

        let power = base.pow(exponent.0)?.0;
        msg!("Calc out given in: power {:?}", power);

        // Assuming complement is 1 - power, but need to adjust for Decimal/BigInt handling
        let one = BigInt::from(1u64 * 10u64.pow(18)); // Fixed point adjustment
        let complement = &one.checked_sub(&power).unwrap();
        //msg!("complement {:?}", complement);

        let amount_out = balance_out
            .checked_mul(&complement)
            .unwrap()
            .checked_div(&BigInt::from(1u64 * 10u64.pow(18)))
            .unwrap(); // Adjust back after fixed point multiplication // Adjust back after fixed point multiplication

        Ok(amount_out)
    }

    fn calc_in_given_out(
        balance_in: &BigInt,
        weight_in: &BigInt,
        balance_out: &BigInt,
        weight_out: &BigInt,
        amount_out: &BigInt,
    ) -> Result<BigInt, MathError> {
        // Ensure amountOut does not exceed balanceOut * MAX_OUT_RATIO

        //let MAX_OUT_RATIO: u32 = 10; // Example value, adjust as needed
        let FIXED_POINT_ONE: BigInt = BigInt::from(1u64 * 10u64.pow(18));

        /*let max_out_ratio = balance_out * &MAX_OUT_RATIO;
        if amount_out > &max_out_ratio {
            return Err(MathError::DividedByZero);
        }*/

        let base = Decimal::div_up(
            Decimal::new(balance_out.clone()),
            Decimal::new(balance_out - amount_out),
        )?;
        let exponent = Decimal::div_up(
            Decimal::new(weight_out.clone()),
            Decimal::new(weight_in.clone()),
        )?;
        let power = Decimal::pow_up(&base.0, &exponent.0)?;

        // Assuming ratio is power - 1, adjusted for Decimal/BigInt handling
        let ratio = &power - &FIXED_POINT_ONE;
        let amount_in = balance_in
            .checked_mul(&ratio)
            .unwrap()
            .checked_div(&FIXED_POINT_ONE)
            .unwrap();

        Ok(amount_in)
    }
    // used to exit the pool inputing the right amount of BP tokens
    fn compute_proportional_amounts_out(
        balances: &[BigInt],
        bpt_total_supply: &BigInt,
        bpt_amount_in: &BigInt,
    ) -> Result<Vec<BigInt>, MathError> {
        // Calculate the ratio of BPT amount in to the total supply, rounding down
        let bpt_ratio = Decimal::div_down(
            Decimal(bpt_amount_in.clone()),
            Decimal(bpt_total_supply.clone()),
        )?;

        // Initialize the vector to hold the calculated amounts out as u64
        let mut amounts_out: Vec<BigInt> = Vec::with_capacity(balances.len());

        // Calculate the proportional amount out for each balance
        for balance in balances {
            let amount_out_decimal =
                Decimal::mul_down(Decimal(balance.clone()), bpt_ratio.clone())?;
            let amount_out_bint = amount_out_decimal.0;

            amounts_out.push(amount_out_bint);
        }

        Ok(amounts_out)
    }

    /*

     fn calc_token_in_given_exact_bpt_out(balance: u128, normalized_weight: u128, bpt_amount_out: u128, bpt_total_supply: u128, swap_fee_percentage: u128) -> Result<U128, MathError> {
         let balance = U128::from(balance);
         let normalized_weight = U128::from(normalized_weight);
         let bpt_amount_out = U128::from(bpt_amount_out);
         let bpt_total_supply = U128::from(bpt_total_supply);
         let swap_fee_percentage = U128::from(swap_fee_percentage);

         let term = bpt_total_supply.checked_div(bpt_total_supply.checked_add(bpt_amount_out).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?;
         let amount_in = balance.checked_mul(U128::one().checked_sub(term.checked_pow(normalized_weight.into()).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?.checked_add(swap_fee_percentage).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?;

         Ok(amount_in.into())
     }


     fn calc_bpt_in_given_exact_tokens_out(balances: &[u128], normalized_weights: &[u128], amounts_out: &[u128], bpt_total_supply: u128, swap_fee_percentage: u128) -> Result<U128, MathError> {
         if balances.len() != normalized_weights.len() || balances.len() != amounts_out.len() {
             return Err(MathError::MulOverflow); // Or a more specific error for mismatched lengths
         }

         let bpt_total_supply = U128::from(bpt_total_supply);
         let swap_fee_percentage = U128::from(swap_fee_percentage);

         let mut total_bpt_in = U128::zero();

         for i in 0..balances.len() {
             let balance = U128::from(balances[i]);
             let normalized_weight = U128::from(normalized_weights[i]);
             let amount_out = U128::from(amounts_out[i]);

             let token_bpt_in = balance
                 .checked_sub(amount_out).ok_or(MathError::DividedByZero)?
                 .checked_div(
                     balance
                         .checked_mul(U128::one().checked_sub(normalized_weight.checked_mul(swap_fee_percentage).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?
                 ).ok_or(MathError::DividedByZero)?;
             total_bpt_in = total_bpt_in.checked_add(token_bpt_in).ok_or(MathError::DividedByZero)?;
         }

         let bpt_in = total_bpt_in.checked_mul(bpt_total_supply).ok_or(MathError::DividedByZero)?;
         Ok(bpt_in.into())
     }
     fn calc_token_out_given_exact_bpt_in(balance: u128, normalized_weight: u128, bpt_amount_in: u128, bpt_total_supply: u128, swap_fee_percentage: u128) -> Result<U128, MathError> {
         let balance = U128::from(balance);
         let normalized_weight = U128::from(normalized_weight);
         let bpt_amount_in = U128::from(bpt_amount_in);
         let bpt_total_supply = U128::from(bpt_total_supply);
         let swap_fee_percentage = U128::from(swap_fee_percentage);

         let term = bpt_total_supply.checked_add(bpt_amount_in.checked_mul(U128::from(1).checked_sub(swap_fee_percentage).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?;
         let token_out = balance.checked_mul(U128::from(1).checked_sub(bpt_total_supply.checked_div(term).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?.checked_pow(normalized_weight).ok_or(MathError::DividedByZero)?).ok_or(MathError::DividedByZero)?;

         Ok(token_out)
     }

     fn calc_tokens_out_given_exact_bpt_in(balances: &[u128], bpt_amount_in: u128, total_bpt: u128) -> Result<Vec<u128>, MathError> {
         let bpt_amount_in = U128::from(bpt_amount_in);
         let total_bpt = U128::from(total_bpt);
         let ratio = bpt_amount_in.checked_div(total_bpt).ok_or(MathError::DividedByZero)?;

         balances.iter().map(|&balance| {
             let balance = U128::from(balance);
             balance.checked_mul(ratio)
                 .map(|res|
                     // Instead of `res.into()`, use the appropriate method to convert `U128` to `u128`.
                     // This example uses `to_u128()` as a placeholder. Replace it with the actual method name.
                     res.as_u128()
                 )
                 .ok_or(MathError::MulOverflow) // Handle the case where `checked_mul` returns None
         }).collect::<Result<Vec<u128>, MathError>>()
     }

    fn calc_due_token_protocol_swap_fee_amount(balance: u128, normalized_weight: u128, previous_invariant: u128, current_invariant: u128, protocol_swap_fee_percentage: u128) -> Result<U128, MathError> {
         let balance = U128::from(balance);
         let normalized_weight = U128::from(normalized_weight);
         let previous_invariant = U128::from(previous_invariant);
         let current_invariant = U128::from(current_invariant);
         let protocol_swap_fee_percentage = U128::from(protocol_swap_fee_percentage);

         if current_invariant <= previous_invariant {
             return Ok(U128::from(0));
         }

         let growth_factor = current_invariant.checked_div(previous_invariant).ok_or(MathError::DividedByZero)?;
         let powered_growth = growth_factor.checked_pow(normalized_weight).ok_or(MathError::DividedByZero)?;
         let fee_base = U128::from(1).checked_sub(powered_growth).ok_or(MathError::DividedByZero)?;
         let due_fee = balance.checked_mul(fee_base).ok_or(MathError::DividedByZero)?.checked_mul(protocol_swap_fee_percentage).ok_or(MathError::DividedByZero)?;

         Ok(due_fee)
     }*/
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_maths::MathError;

    #[test]
    fn test_calculate_invariant_basic() {
        let normalized_weights = [500_000_000u128, 500_000_000u128]; // 50% each, assuming full precision
        let balances = [1000u128, 1000u128]; // Equal balances for simplicity
        let result = calculate_invariant(&normalized_weights, &balances)
            .expect("Invariant calculation failed");
        assert!(result > U128::from(0), "Invariant should be greater than 0");
    }

    #[test]
    fn test_calculate_invariant_with_precision() {
        let normalized_weights = [700_000_000u128, 300_000_000u128]; // 70% - 30% split
        let balances = [700u128, 300u128];
        let result = calculate_invariant(&normalized_weights, &balances)
            .expect("Invariant calculation failed");
        // Adjust the expected result based on actual function logic and precision handling
        assert!(result > U128::from(0), "Invariant should be greater than 0");
    }

    #[test]
    fn test_calculate_invariant_error_handling() {
        let normalized_weights = [0u128]; // Invalid input
        let balances = [1000u128];
        let result = calculate_invariant(&normalized_weights, &balances);
        assert!(
            matches!(result, Err(MathError::MulOverflow)),
            "Expected precision loss error"
        );
    }
}
