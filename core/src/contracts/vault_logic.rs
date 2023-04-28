use ethabi::ethereum_types::U256;
use rust_decimal::Decimal;

use crate::log;

use super::{token::{Token, self}, vault::VaultState};
use lazy_static::lazy_static;

const USDP_DECIMALS: u32 = 18;
const MINT_BURN_FEE_BASIS_POINTS: u32 = 0;
const TAX_BASIS_POINTS: u32 = 0;

lazy_static! {
    static ref PRECISION: U256 = U256::from(1) * U256::from(10u32).pow(30.into());
    static ref BASIS_POINTS_DIVISOR: U256 = U256::from(10000);
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
    fn get_swap_details(&self, token_in: &Token, token_out: &Token, amount_in: U256) -> (U256, U256, u64);
    fn get_fee_basis_points_swap(
        &self,
        is_stable_coin_swap: bool,
        token_weight: u64,
        token_usdg_amount: &U256,
        usdp_delta: &U256,
        increment: bool,
    ) -> u32;
}


impl VaultLogic for VaultState {
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
        get_fee_basis_points(
            token_weight,
            token_usdg_amount,
            usdp_delta,
            self.mint_burn_fee_basis_points,
            U256::from(self.tax_basis_points),
            increment,
            &self.usdp_supply,
            &self.total_token_weights,
            self.has_dynamic_fees,
        )
    }

    fn get_fee_basis_points_swap(
        &self,
        is_stable_coin_swap: bool,
        token_weight: u64,
        token_usdg_amount: &U256,
        usdp_delta: &U256,
        increment: bool,
    ) -> u32 {
        let base_bps = if is_stable_coin_swap {
            self.stable_swap_fee_basis_points
        } else {
            self.swap_fee_basis_points
        };
        let tax_bps = if is_stable_coin_swap {
            self.stable_tax_basis_points
        } else {
            U256::from(self.tax_basis_points)
        };
        
        get_fee_basis_points(
            token_weight,
            token_usdg_amount,
            usdp_delta,
            base_bps,
            U256::from(tax_bps),
            increment,
            &self.usdp_supply,
            &self.total_token_weights,
            self.has_dynamic_fees,
        )
    }

    fn get_buy_glp_to_amount(
        &self,
        from_amount: &U256,
        pay_token: &Token,
        // plp_price: &U256,
        // usdp_supply: &U256,
        // total_token_weights: &U256,
    ) -> (U256, u64) {
        get_buy_glp_to_amount(
            from_amount,
            pay_token,
            &self.get_plp_price(true),
            &self.usdp_supply,
            &self.total_token_weights,
            self.has_dynamic_fees,
            self.mint_burn_fee_basis_points,
            U256::from(self.tax_basis_points),
        )
    }

    fn get_sell_glp_from_amount(
        &self,
        to_amount: U256,
        from_token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64) {
        get_sell_glp_from_amount(
            to_amount,
            from_token,
            self.get_plp_price(false),
            self.usdp_supply,
            self.total_token_weights,
            self.has_dynamic_fees,
            self.mint_burn_fee_basis_points,
            U256::from(self.tax_basis_points),
        )
    }

    fn get_buy_glp_from_amount(
        &self,
        to_amount: U256,
        token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64) {
        get_buy_glp_from_amount(
            to_amount,
            token,
            self.get_plp_price(true),
            self.usdp_supply,
            self.total_token_weights, self.has_dynamic_fees,
            self.mint_burn_fee_basis_points,
            U256::from(self.tax_basis_points),
        )
    }

    fn get_sell_glp_to_amount(
        &self,
        to_amount: U256,
        from_token: &Token,
        // plp_price: U256,
        // usdp_supply: U256,
        // total_token_weights: U256,
    ) -> (U256, u64) {
        get_sell_glp_to_amount(
            to_amount,
            from_token,
            self.get_plp_price(false),
            self.usdp_supply,
            self.total_token_weights,
            self.has_dynamic_fees,
            self.mint_burn_fee_basis_points,
            U256::from(self.tax_basis_points),
        )
    }

    fn get_plp_price(&self, is_buy: bool) -> U256 {
        let aum = if is_buy { self.total_aum[0] } else { self.total_aum[1] };
        if self.usdp_supply.eq(&U256::from(0)) {
            return U256::from(0);
        } else {
            (aum * expand_decimals(1, 18)) / (self.plp_supply * expand_decimals(1, 12))
        }
    }

    fn get_swap_details(&self, token_in: &Token, token_out: &Token, amount_in: U256) -> (U256, U256, u64) {
        let price_in = token_in.ask_price.expect("Invalid ask price");
        let price_out = token_out.bid_price.expect("Invalid bid price");
        let mut amount_out = amount_in * price_in.raw / price_out.raw;  
        amount_out = adjust_for_decimals(&amount_out, token_in.decimals.into(), token_out.decimals.into());

        let mut usdp_amount = amount_in * price_in.raw / *PRECISION;
        usdp_amount = adjust_for_decimals(&usdp_amount, token_in.decimals.into(), token_out.decimals.into());
        let is_stable_coin_swap = token_in.is_stable_token.unwrap() && token_out.is_stable_token.unwrap();

        let fee_bps0 = self.get_fee_basis_points_swap(
            is_stable_coin_swap,
            token_in.token_weight.expect("Invalid token weight"),
            &token_in.usdp_amount.expect("Invalid usdp amount"),
            &usdp_amount,
            true,
        );
        let fee_bps1 = self.get_fee_basis_points_swap(
            is_stable_coin_swap,
            token_out.token_weight.expect("Invalid token weight"),
            &token_out.usdp_amount.expect("Invalid usdp amount"),
            &usdp_amount,
            false
        );
        let fee_bps = if fee_bps0 > fee_bps1 { fee_bps0 } else { fee_bps1 };

        let amount_out_after_fee = amount_out * (*BASIS_POINTS_DIVISOR - fee_bps) / (*BASIS_POINTS_DIVISOR);
        log::print(format!("amount_out: {}, amount_out_after_fee: {}", amount_out, amount_out_after_fee).as_str());
        let fee_amount = amount_out - amount_out_after_fee;
        (amount_out, fee_amount, fee_bps.into())
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


fn get_fee_basis_points(
    token_weight: u64,
    token_usdg_amount: &U256,
    usdp_delta: &U256,
    fee_basis_points: U256,
    tax_basis_points: U256,
    increment: bool,
    usdp_supply: &U256,
    total_token_weights: &U256,
    has_dynamic_fees: bool,
) -> u32 {
    println!("************ get_fee_basis_points ***********");
    println!("token_usdg_amount {} ", token_usdg_amount);
    println!("usdp_supply {} ", usdp_supply);
    println!("total_token_weights {} ", total_token_weights);
    println!("has_dynamic_fees {} ", has_dynamic_fees);
    if token_usdg_amount.is_zero() || usdp_supply.is_zero() || total_token_weights.is_zero() {
        return 0;
    }

    if !has_dynamic_fees {
        return fee_basis_points.as_u32();
    }
    // println!()

    let initial_amount = token_usdg_amount.clone();

    println!("increment {}", increment);
    println!("initial_amount {}", initial_amount);
    println!("usdp_delta {}", usdp_delta);
    let next_amount = if increment {
        initial_amount.clone() + usdp_delta
    } else {
        if usdp_delta > &initial_amount {
            U256::zero()
        } else {
            initial_amount.clone() - usdp_delta
        }
    };

    println!("nex_amount {}", next_amount);

    let target_amount = get_target_usdg_amount(token_weight, usdp_supply, total_token_weights).expect("No target amount");
    println!("target_amount {}", target_amount);

    if target_amount.is_zero() {
        return fee_basis_points.low_u32();
    }

    let initial_diff = if initial_amount > target_amount {
        initial_amount - target_amount
    } else {
        target_amount - initial_amount
    };
    println!("initial_diff {}", initial_diff);

    let next_diff = if next_amount > target_amount {
        next_amount - target_amount
    } else {
        target_amount - next_amount
    };

    if next_diff < initial_diff {
        let rebate_bps = tax_basis_points.clone() * initial_diff.clone() / target_amount.clone();
        println!("rebate_bps {}", rebate_bps);

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


// Buy PLP - token to exact token (PLP)
pub fn get_buy_glp_from_amount(
    to_amount: U256,
    token: &Token,
    plp_price: U256,
    usdp_supply: U256,
    total_token_weights: U256,
    has_dynamic_fees: bool,
    fee_basis_points: U256,
    tax_fee_basis_points: U256,
) -> (U256, u64) {
    if to_amount == U256::zero()
        || usdp_supply == U256::zero()
        || total_token_weights == U256::zero()
    {
        return (U256::zero(), 0);
    }
    let default_value = (U256::zero(), 0);

    let min_price = token.min_price.clone().expect("no min price").raw;

    let mut from_amount = to_amount * plp_price / min_price;

    println!("from_amount: {}", from_amount);


    // from_amount = adjust_for_decimals(&from_amount, USDP_DECIMALS, token.decimals.into());

    println!("from_amount adjust_for_decimals: {}", from_amount);



    let mut usdg_amount = to_amount * plp_price * expand_decimals(1, 18) / *PRECISION;

    println!("usdg_amount: {}", usdg_amount);

    usdg_amount = adjust_for_decimals(&usdg_amount, token.decimals.into(), USDP_DECIMALS);


    let fee_basis_points = get_fee_basis_points(
        token.token_weight.unwrap_or(0),
        &token.usdp_amount.unwrap(),
        &usdg_amount,
        fee_basis_points,
        tax_fee_basis_points,
        true,
        &usdp_supply,
        &total_token_weights,
        has_dynamic_fees,
    );


    from_amount = from_amount * *BASIS_POINTS_DIVISOR / (*BASIS_POINTS_DIVISOR - fee_basis_points);

    (from_amount, fee_basis_points.into())
}

// Buy PLP - exact token to token (PLP)
pub fn get_buy_glp_to_amount(
    from_amount: &U256,
    pay_token: &Token,
    plp_price: &U256,
    usdp_supply: &U256,
    total_token_weights: &U256,
    has_dynamic_fees: bool,
    fee_basis_points: U256,
    tax_fee_basis_points: U256,
) -> (U256, u64) {
    if from_amount == &U256::zero()
        || usdp_supply == &U256::zero()
        || total_token_weights == &U256::zero()
    {
        return (U256::zero(), 0);
    }
    let default_value = (U256::zero(), 0);
    if from_amount.is_zero()
        || plp_price.is_zero()
        || usdp_supply.is_zero()
        || total_token_weights.is_zero()
    {
        return default_value;
    }
    //
    let min_price = pay_token.min_price.clone().expect("no min price").raw;


    // let pay_token = get_token_info(info_tokens, swap_token_address);
    if min_price.is_zero() {
        return default_value;
    }

    let mut glp_amount = from_amount
        .checked_mul(min_price )
        .unwrap()
        .checked_div(*plp_price)
        .unwrap();


    println!("pay_token.decimals.into(): {}, glp_amount: {}", pay_token.decimals, glp_amount);

    // glp_amount = adjust_for_decimals(&glp_amount,  USDP_DECIMALS,pay_token.decimals.into());

    println!("glp_amount: {}", glp_amount);


    let mut usdg_amount = from_amount
        .checked_mul(min_price)
        .unwrap()
        .checked_div(*PRECISION)
        .unwrap();
    println!("usdg_amount: {}", usdg_amount);

    usdg_amount = adjust_for_decimals(&usdg_amount, pay_token.decimals.into(), USDP_DECIMALS);


    // const MINT_BURN_FEE_BASIS_POINTS: u32 = 10;
    // const TAX_BASIS_POINTS: u32 = 10;
    let fee_basis_points = get_fee_basis_points(
        pay_token.token_weight.unwrap(),
        &pay_token.usdp_amount.unwrap(),
        &usdg_amount,
        fee_basis_points,
        tax_fee_basis_points,
        true,
        usdp_supply,
        total_token_weights,
        has_dynamic_fees,
    );

    glp_amount = glp_amount
        .checked_mul(*BASIS_POINTS_DIVISOR - fee_basis_points)
        .unwrap()
        .checked_div(*BASIS_POINTS_DIVISOR)
        .unwrap();

    (glp_amount, fee_basis_points.into())
}


// Sell PLP- token  (PLP) to exact token
pub fn get_sell_glp_from_amount(
    to_amount: U256,
    swap_token: &Token,
    plp_price: U256,
    usdp_supply: U256,
    total_token_weights: U256,
    has_dynamic_fees: bool,
    fee_basis_points: U256,
    tax_fee_basis_points: U256,
) -> (U256, u64) {
    if to_amount == U256::zero()
        || usdp_supply == U256::zero()
        || total_token_weights == U256::zero()
    {
        return (U256::zero(), 0);
    }
    let max_price = swap_token.max_price.clone().expect("no max price").raw;

    println!("max_price {}", max_price);

    if max_price == U256::zero() {
        return (U256::zero(), 0);
    }

    let mut glp_amount = to_amount * max_price / plp_price;

    println!("glp_amount {}", glp_amount);
    // glp_amount = adjust_for_decimals(&glp_amount, swap_token.decimals.into(), USDP_DECIMALS);

    println!("glp_amount adjust_for_decimals {}", glp_amount);


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
        fee_basis_points,
        tax_fee_basis_points,
        false,
        &usdp_supply,
        &total_token_weights,
        has_dynamic_fees,
    );

    glp_amount = glp_amount * *BASIS_POINTS_DIVISOR / (*BASIS_POINTS_DIVISOR - fee_basis_points);

    (glp_amount, fee_basis_points.into())
}

// Sell PLP- exact token  (PLP) to token
pub fn get_sell_glp_to_amount(
    to_amount: U256,
    from_token: &Token,
    plp_price: U256,
    usdp_supply: U256,
    total_token_weights: U256,
    has_dynamic_fees: bool,
    fee_basis_points: U256,
    tax_fee_basis_points: U256,
) -> (U256, u64) {
    if to_amount == U256::zero()
        || usdp_supply == U256::zero()
        || total_token_weights == U256::zero()
    {
        return (U256::zero(), 0);
    }
    let default_value = (U256::zero(), 0);

    let max_price = from_token.max_price.clone().expect("no max price").raw;

    println!("max_price {}", max_price);
    println!("to_amount {}", to_amount);
    println!("plp_price {}", plp_price);

    let mut from_amount = to_amount * plp_price  / max_price;

    println!("from_amount {}", from_amount);

    // from_amount = adjust_for_decimals(&from_amount, USDP_DECIMALS, from_token.decimals.into());
    println!("from_amount adjust_for_decimals {}", from_amount);


    let usdg_amount = to_amount * plp_price * expand_decimals(1, 18) / *PRECISION;
    println!("usdg_amount {}", usdg_amount);


    // In the Vault contract, the USDG supply is reduced before the fee basis points are calculated
    let new_usdg_supply = usdp_supply.clone().checked_sub(usdg_amount).unwrap_or(U256::zero());

    println!("new_usdg_supply {}", new_usdg_supply);


    // In the Vault contract, the token.usdg_amount is reduced before the fee basis points are calculated
    let fee_basis_points = get_fee_basis_points(
        from_token.token_weight.unwrap_or(0),
        &from_token.usdp_amount.unwrap_or(U256::from(0)).checked_sub(usdg_amount).unwrap_or(U256::zero()),
        &usdg_amount,
        fee_basis_points,
        tax_fee_basis_points,
        false,
        &new_usdg_supply,
        &total_token_weights,
        has_dynamic_fees,
    );

    println!("fee_basis_points {}", fee_basis_points);
    println!("from_amount 2 {}", from_amount);


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

