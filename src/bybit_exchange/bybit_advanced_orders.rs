#[derive(Debug, Eq, PartialEq)]
pub enum OrderEntryType {
	Market,
	Limit,
    //this exists so other order types can be added in the future
}

