#[derive(Debug, Eq, PartialEq)]
pub enum OrderEntryType {
	Market,
	Limit,
	Conditional,
	OrderBook,
    //this exists so other order types can be added in the future
}

