
use ethabi::ethereum_types::U256;

use super::token::Token;
use lazy_static::lazy_static;

const USDP_DECIMALS: u32 = 18;

lazy_static! {
    static ref PRECISION: U256 = U256::from(0);
    static ref BASIS_POINTS_DIVISOR: U256 = U256::from(0);
}


pub trait VaultLogic {
    fn get_fee_basis_points(
        &self,
        token_weight: u64,
        token_usdg_amount: &U256,
        usdg_delta: &U256,
        fee_basis_points: u32,
        tax_basis_points: u32,
        increment: bool,
        usdg_supply: &U256,
        total_token_weights: &U256,
    ) -> u32;
    
}

fn adjust_for_decimals(amount: &U256, div_decimals: u32, mul_decimals: u32) -> U256 {
    amount * expand_decimals(1, mul_decimals) / expand_decimals(1, div_decimals)
}

fn get_target_usdg_amount(token_weight: u64, usdg_supply: &U256, total_token_weights: &U256) -> Option<U256> {
    let token_weight = U256::from(token_weight);
    if token_weight.is_zero() || usdg_supply.is_zero() {
        return None;
    }

    Some(token_weight * usdg_supply / total_token_weights)
}

pub fn get_buy_glp_to_amount(
    from_amount: &U256,
    pay_token: &Token,
    glp_price: &U256,
    usdg_supply: &U256,
    total_token_weights: &U256,
) -> (U256, u32) {
    let default_value = (U256::zero(), 0);
    if from_amount.is_zero()
        || glp_price.is_zero()
        || usdg_supply.is_zero()
        || total_token_weights.is_zero()
    {
        return default_value;
    }

    let min_price = pay_token.min_price.clone().expect("no min price").raw;

    // let pay_token = get_token_info(info_tokens, swap_token_address);
    if min_price.is_zero() {
        return default_value;
    }

    let mut glp_amount = from_amount
        .checked_mul(min_price)
        .unwrap()
        .checked_div(*glp_price)
        .unwrap();
    glp_amount = adjust_for_decimals(&glp_amount, pay_token.decimals.into(), USDP_DECIMALS);

    let mut usdg_amount = from_amount
        .checked_mul(min_price)
        .unwrap()
        .checked_div(*PRECISION)
        .unwrap();
    usdg_amount = adjust_for_decimals(&usdg_amount, pay_token.decimals.into(), USDP_DECIMALS);
    const MINT_BURN_FEE_BASIS_POINTS: u32 = 10;
    const TAX_BASIS_POINTS: u32 = 10;
    let fee_basis_points = get_fee_basis_points(
        pay_token.token_weight.unwrap(),
        &pay_token.usdp_amount.unwrap(),
        &usdg_amount,
        MINT_BURN_FEE_BASIS_POINTS,
        TAX_BASIS_POINTS,
        true,
        usdg_supply,
        total_token_weights
    );

    glp_amount = glp_amount
        .checked_mul(*BASIS_POINTS_DIVISOR - fee_basis_points)
        .unwrap()
        .checked_div(*BASIS_POINTS_DIVISOR)
        .unwrap();

    (glp_amount, fee_basis_points)
}



fn get_fee_basis_points(
    token_weight: u64,
    token_usdg_amount: &U256,
    usdg_delta: &U256,
    fee_basis_points: u32,
    tax_basis_points: u32,
    increment: bool,
    usdg_supply: &U256,
    total_token_weights: &U256,
) -> u32 {
    if token_usdg_amount.is_zero() || usdg_supply.is_zero() || total_token_weights.is_zero() {
        return 0;
    }

    let fee_basis_points = U256::from(fee_basis_points);
    let tax_basis_points = U256::from(tax_basis_points);

    let initial_amount = token_usdg_amount.clone();
    let next_amount = if increment {
        initial_amount.clone() + usdg_delta
    } else {
        if usdg_delta > &initial_amount {
            U256::zero()
        } else {
            initial_amount.clone() - usdg_delta
        }
    };

    let target_amount = get_target_usdg_amount(token_weight, usdg_supply, total_token_weights).expect("No target amount");
    if target_amount.is_zero() {
        return fee_basis_points.low_u32();
    }

    let initial_diff = if initial_amount > target_amount {
        initial_amount - target_amount
    } else {
        target_amount - initial_amount
    };
    let next_diff = if next_amount > target_amount {
        next_amount - target_amount
    } else {
        target_amount - next_amount
    };

    if next_diff < initial_diff {
        let rebate_bps = tax_basis_points.clone() * initial_diff.clone() / target_amount.clone();
        if rebate_bps > fee_basis_points {
            0
        } else {
            (fee_basis_points.clone() - rebate_bps).low_u32()
        }
    } else {
        let mut average_diff = (initial_diff.clone() + next_diff.clone()) / 2;
        if average_diff > target_amount {
            average_diff = target_amount.clone();
        }
        let tax_bps = tax_basis_points.clone() * average_diff.clone() / target_amount.clone();
        (fee_basis_points.clone() + tax_bps).low_u32()
    }
}

fn expand_decimals(value: u32, decimals: u32) -> U256 {
    U256::from(value) * U256::from(10u32).pow(decimals.into())
}

