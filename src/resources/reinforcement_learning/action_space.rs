use bevy::prelude::Resource;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum CompanyAction {
    Nothing,
    BuyProcessor(usize),
    SellProcessor(usize),
    BuyResource(usize, usize),
    SellResource(usize, usize),
}

#[derive(Resource, Default)]
pub struct ActionSpace {
    pub actions: Vec<CompanyAction>,
}

impl ActionSpace {
    pub fn new(resource_count: usize, recipe_count: usize) -> ActionSpace {
        let mut actionspace: Vec<CompanyAction> = Vec::new();
        actionspace.push(CompanyAction::Nothing);
        for i in 0..recipe_count {
            actionspace.push(CompanyAction::BuyProcessor(i));
        }
        for i in 0..recipe_count {
            actionspace.push(CompanyAction::SellProcessor(i));
        }
        // TODO: Allow creation of offers / orders that are not according to best price policy
        for i in 0..resource_count {
            actionspace.push(CompanyAction::BuyResource(i, 1));
            actionspace.push(CompanyAction::SellResource(i, 1));
        }
        ActionSpace {
            actions: actionspace,
        }
    }
}
