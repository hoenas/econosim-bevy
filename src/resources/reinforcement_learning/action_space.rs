use bevy::prelude::Resource;

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
        // TODO: Allow creation of offers / orders that are not according to best price policy
        for i in 0..resource_count {
            actionspace.push(CompanyActionEnum::BuyResource(i, 1));
            actionspace.push(CompanyActionEnum::SellResource(i, 1));
        }
        ActionSpace {
            actions: actionspace,
        }
    }
}
