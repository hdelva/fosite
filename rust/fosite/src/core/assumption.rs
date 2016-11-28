use super::GastID;

#[derive(Debug, Clone)]
pub enum Assumption {
    None,
    ConditionAssumption {
        source: GastID,
        negated: bool
    },
    Multiple (
        Vec<Assumption>
    ),
}