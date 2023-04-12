
use ethabi::ethereum_types::U256;
use rust_decimal::Decimal;

use super::{token::Token, vault::VaultState};
use lazy_static::lazy_static;

const USDP_DECIMALS: u32 = 18;
const MINT_BURN_FEE_BASIS_POINTS: u32 = 0;
const TAX_BASIS_POINTS: u32 = 0;

lazy_static! {
    static ref PRECISION: U256 = U256::from(0);
    static ref BASIS_POINTS_DIVISOR: U256 = U256::from(0);
}

pub trait VaultLogic {
    fn get_fee_basis_points(
        &self,
        token_weight: u64,
        token_usdg_amount: &U256,
        usdp_delta: &U256,
        // fee_basis_points: u32,
        // tax_basis_points: u32,
        increment: bool,
        // usdp_supply: &U256,
        // total_token_weights: &U256,
    ) -> u32;
    fn get_buy_glp_to_amount(
        &self,
        from_amount: &U256,
        pay_token: &Token,
        // plp_price: &U256,
        // usdp_supply: &U256,
        // total_token_weights: &U256,
    ) -> (U256, u64);
    fn get_sell_glp_from_amount(
        &self,
        to_amount: U256,
        from_token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64);
    fn get_buy_glp_from_amount(
        &self,
        to_amount: U256,
        token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64);
    fn get_sell_glp_to_amount(
        &self,
        to_amount: U256,
        from_token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64);
    fn get_plp_price(&self, is_buy: bool) -> U256;
}


impl VaultLogic for VaultState  {
    fn get_fee_basis_points(
        &self,
        token_weight: u64,
        token_usdg_amount: &U256,
        usdp_delta: &U256,
        // fee_basis_points: u32,
        // tax_basis_points: u32,
        increment: bool,
        // usdp_supply: &U256,
        // total_token_weights: &U256,
    ) -> u32 {
        get_fee_basis_points(token_weight, token_usdg_amount, usdp_delta, self.fee_basis_points, self.tax_basis_points, increment, &self.usdp_supply, &self.total_token_weights)
    }

    fn get_buy_glp_to_amount(
        &self,
        from_amount: &U256,
        pay_token: &Token,
        // plp_price: &U256,
        // usdp_supply: &U256,
        // total_token_weights: &U256,
    ) -> (U256, u64) {
        get_buy_glp_to_amount(from_amount, pay_token, &self.get_plp_price(true), &self.usdp_supply, &self.total_token_weights)
    }

    fn get_sell_glp_from_amount(
        &self,
        to_amount: U256,
        from_token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64) {
        get_sell_glp_to_amount(to_amount, from_token, self.get_plp_price(false), self.usdp_supply, self.total_token_weights)
    }

    fn get_buy_glp_from_amount(
        &self,
        to_amount: U256,
        token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64) {
        get_buy_glp_from_amount(to_amount, token, self.get_plp_price(true), self.usdp_supply, self.total_token_weights)
    }

    fn get_sell_glp_to_amount(
        &self,
        to_amount: U256,
        from_token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64) {
        get_sell_glp_to_amount(to_amount, from_token, self.get_plp_price(false), self.usdp_supply, self.total_token_weights)
    }

    fn get_plp_price(&self, is_buy: bool) -> U256 {
        let aum = if is_buy {self.total_aum[0]} else {self.total_aum[1]};
        if self.usdp_supply.eq(&U256::from(0)) {
            return U256::from(0);
        }
        else{
            aum / self.plp_supply
        }
    }
}


fn adjust_for_decimals(amount: &U256, div_decimals: u32, mul_decimals: u32) -> U256 {
    amount * expand_decimals(1, mul_decimals) / expand_decimals(1, div_decimals)
}

fn get_target_usdg_amount(token_weight: u64, usdp_supply: &U256, total_token_weights: &U256) -> Option<U256> {
    let token_weight = U256::from(token_weight);
    if token_weight.is_zero() || usdp_supply.is_zero() {
        return None;
    }

    Some(token_weight * usdp_supply / total_token_weights)
}

fn get_buy_glp_to_amount(
    from_amount: &U256,
    pay_token: &Token,
    plp_price: &U256,
    usdp_supply: &U256,
    total_token_weights: &U256,
) -> (U256, u64) {
    let default_value = (U256::zero(), 0);
    if from_amount.is_zero()
        || plp_price.is_zero()
        || usdp_supply.is_zero()
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
        .checked_div(*plp_price)
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
        usdp_supply,
        total_token_weights
    );

    glp_amount = glp_amount
        .checked_mul(*BASIS_POINTS_DIVISOR - fee_basis_points)
        .unwrap()
        .checked_div(*BASIS_POINTS_DIVISOR)
        .unwrap();

    (glp_amount, fee_basis_points.into())
}



fn get_fee_basis_points(
    token_weight: u64,
    token_usdg_amount: &U256,
    usdp_delta: &U256,
    fee_basis_points: u32,
    tax_basis_points: u32,
    increment: bool,
    usdp_supply: &U256,
    total_token_weights: &U256,
) -> u32 {
    if token_usdg_amount.is_zero() || usdp_supply.is_zero() || total_token_weights.is_zero() {
        return 0;
    }

    let fee_basis_points = U256::from(fee_basis_points);
    let tax_basis_points = U256::from(tax_basis_points);

    let initial_amount = token_usdg_amount.clone();
    let next_amount = if increment {
        initial_amount.clone() + usdp_delta
    } else {
        if usdp_delta > &initial_amount {
            U256::zero()
        } else {
            initial_amount.clone() - usdp_delta
        }
    };

    let target_amount = get_target_usdg_amount(token_weight, usdp_supply, total_token_weights).expect("No target amount");
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

fn get_sell_glp_from_amount(
    to_amount: U256,
    swap_token: &Token,
    plp_price: U256,
    usdp_supply: U256,
    total_token_weights: U256,
) -> (U256, u64) {
    if to_amount == U256::zero()
        || usdp_supply == U256::zero()
        || total_token_weights == U256::zero()
    {
        return (U256::zero(), 0);
    }
    let max_price = swap_token.max_price.clone().expect("no max price").raw;

    if max_price == U256::zero() {
        return (U256::zero(), 0);
    }

    let mut glp_amount = to_amount * max_price / plp_price;
    glp_amount = adjust_for_decimals(&glp_amount, swap_token.decimals.into(), USDP_DECIMALS);

    let mut usdg_amount = to_amount * max_price / *PRECISION;
    usdg_amount = adjust_for_decimals(&usdg_amount, swap_token.decimals.into(), USDP_DECIMALS);

    // In the Vault contract, the USDG supply is reduced before the fee basis points are calculated
    let usdp_supply = usdp_supply
        .checked_sub(usdg_amount)
        .unwrap_or(U256::zero());

    let fee_basis_points = get_fee_basis_points(
        swap_token.token_weight.unwrap_or(0),
        &swap_token.usdp_amount.unwrap().checked_sub(usdg_amount).unwrap_or(U256::zero()),
        &usdg_amount,
        MINT_BURN_FEE_BASIS_POINTS,
        TAX_BASIS_POINTS,
        false,
        &usdp_supply,
        &total_token_weights,
    );

    glp_amount = glp_amount * *BASIS_POINTS_DIVISOR / (*BASIS_POINTS_DIVISOR - fee_basis_points);

    (glp_amount, fee_basis_points.into())
}



pub fn get_buy_glp_from_amount(
    to_amount: U256,
    token: &Token,
    plp_price: U256,
    usdp_supply: U256,
    total_token_weights: U256,
) -> (U256, u64) {
    let default_value = (U256::zero(), 0);

    let min_price = token.min_price.clone().expect("no min price").raw;

    let mut from_amount = to_amount * plp_price / min_price;
    from_amount = adjust_for_decimals(&from_amount, USDP_DECIMALS, token.decimals.into());

    let usdg_amount = to_amount * plp_price / *PRECISION;
    let fee_basis_points = get_fee_basis_points(
        token.token_weight.unwrap_or(0),
        &token.usdp_amount.unwrap(),
        &usdg_amount,
        MINT_BURN_FEE_BASIS_POINTS,
        TAX_BASIS_POINTS,
        true,
        &usdp_supply,
        &total_token_weights,
    );

    from_amount = from_amount * *BASIS_POINTS_DIVISOR / (*BASIS_POINTS_DIVISOR - fee_basis_points);

    (from_amount, fee_basis_points.into())
}

pub fn get_sell_glp_to_amount(
    to_amount: U256,
    from_token: &Token,
    plp_price: U256,
    usdp_supply: U256,
    total_token_weights: U256,
) -> (U256, u64) {
    let default_value = (U256::zero(), 0);

    let max_price = from_token.max_price.clone().expect("no max price").raw;

    let mut from_amount = to_amount * plp_price / max_price;
    from_amount = adjust_for_decimals(&from_amount, USDP_DECIMALS, from_token.decimals.into());

    let usdg_amount = to_amount * plp_price / *PRECISION;

    // In the Vault contract, the USDG supply is reduced before the fee basis points are calculated
    let new_usdg_supply = usdp_supply.clone().checked_sub(usdg_amount).unwrap_or(U256::zero());

    // In the Vault contract, the token.usdg_amount is reduced before the fee basis points are calculated
    let fee_basis_points = get_fee_basis_points(
        from_token.token_weight.unwrap_or(0),
        &from_token.usdp_amount.unwrap_or(U256::from(0)).checked_sub(usdg_amount).unwrap_or(U256::zero()),
        &usdg_amount,
        MINT_BURN_FEE_BASIS_POINTS,
        TAX_BASIS_POINTS,
        false,
        &new_usdg_supply,
        &total_token_weights,
    );

    from_amount = from_amount * *BASIS_POINTS_DIVISOR / (*BASIS_POINTS_DIVISOR - fee_basis_points);

    (from_amount, fee_basis_points.into())
}

// Usage example
// fn main() {
//     let mut info_tokens = HashMap::new();
//
//     // Fill the info_tokens with data
//
//     let from_amount = U256::from(1000);
//     let swap_token_address = "0x123...";
//     let plp_price = U256::from(2000);
//     let usdp_supply = U256::from(5000);
//     let total_token_weights = U256::from(8000);
//
//     let (glp_amount, fee_basis_points) = get_buy_glp_to_amount(
//         from_amount,
//         swap_token_address,
//         &info_tokens,
//         plp_price,
//         usdp_supply,
//         total_token_weights,
//     );
//
//     println!("GLP amount: {:?}", glp_amount);
//     println!("Fee basis points: {:?}", fee_basis_points);
// }
//

