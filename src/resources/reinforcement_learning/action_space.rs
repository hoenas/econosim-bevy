use bevy::prelude::Resource;

/// Discrete quantity tiers available for buy/sell orders.
/// One action is generated per resource per tier for both buying and selling.
const TRADE_AMOUNTS: &[usize] = &[10, 100, 1_000, 10_000];

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum CompanyActionEnum {
    Nothing,
    BuyProcessor(usize),
    SellProcessor(usize),
    BuyResource(usize, usize),
    SellResource(usize, usize),
}

#[derive(Resource, Default)]
pub struct ActionSpace {
    pub actions: Vec<CompanyActionEnum>,
}

impl ActionSpace {
    pub fn new(resource_count: usize, recipe_count: usize) -> ActionSpace {
        let mut actionspace: Vec<CompanyActionEnum> = Vec::new();
        actionspace.push(CompanyActionEnum::Nothing);
        for i in 0..recipe_count {
            actionspace.push(CompanyActionEnum::BuyProcessor(i));
        }
        for i in 0..recipe_count {
            actionspace.push(CompanyActionEnum::SellProcessor(i));
        }
        for i in 0..resource_count {
            for &amount in TRADE_AMOUNTS {
                actionspace.push(CompanyActionEnum::BuyResource(i, amount));
                actionspace.push(CompanyActionEnum::SellResource(i, amount));
            }
        }
        ActionSpace {
            actions: actionspace,
        }
    }
}
