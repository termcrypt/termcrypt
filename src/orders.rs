use anyhow::{bail, Error, Result};

// Input variables into order calculation function
#[derive(Debug)]
pub struct OrderCalcEntry {
	// Total available quantity
	pub total_liquid: f64,
	// Trade risk (on SL) in percentage
	pub risk: f64,
	// Trade stoploss price
	pub stoploss: f64,
	// Trade take-profit price
	pub takeprofit: f64,
	// Trade potential entry price
	pub entry: f64,
}

// Calculated variables exiting order calculation
#[derive(Debug)]
pub struct OrderCalcExit {
	// Order quantity accounting for risk and other factors
	pub quantity: f64,
	// Bool for if variable is of order type long or short
	pub islong: bool,
	// Take-profit/Stop-loss ratio (R rate)
	pub tpslratio: f64,
}

// User trade entry type (technical order type)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntryType {
	// Market order type (accepting offer)
	Market,
	// Limit order type (creating offer)
	Limit,
}

// Exchange coin market type (exchange specific)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExchangeContext {
	// Account requires coin itself to place order (such as BTC)
	// Supports leverage
	BybitInverse,
	// Account requires asset coin is paired with to palce order (such as USDT)
	// Supports leverage
	BybitLinear,
	// Does not support leverage
	_BybitSpot,
	// Derivatives that expire at specific dates
	// Supports leverage
	_BybitFutures,
}

// Application-individual order entry types (not technical order type)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderEntryType {
	// Immediate order
	Market,
	// Trigger order at a certain level
	Limit,
	// Not used at the moment...
	Conditional,
	// Orderbook order (opens a limit order at a user specified step difference from mark price)
	OrderBook,
	// More types to come...
}

// Direction type for local trade struct
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderDirection {
	// Profits when price goes up
	Long,
	// Profits when price goes down
	Short,
}

// Calculate order quantities and validate inputs
pub fn calculate_order(ov: OrderCalcEntry) -> Result<OrderCalcExit, Error> {
	let error_margin = f64::EPSILON;

	let mut _islong = true;
	let mut _quantity: f64 = 1.0;
	let mut _tpslratio: f64 = 1.0;

	// Checks if parameters are correct
	// More should be added here for more cases - sometimes invalid data can be passed!

	if (ov.risk > 100.0)
		// Check if inputs are not 0
		|| (ov.stoploss - ov.takeprofit).abs() < error_margin
		|| (ov.stoploss - ov.entry).abs() < error_margin
		|| (ov.takeprofit - ov.entry).abs() < error_margin
		|| ov.risk < 0.1
	{
		bail!("Invalid parameters (error in calculating order size) - Check your inputs");
	}

	// Checks if order is of long type or is of short type
	if ov.stoploss < ov.takeprofit && ov.entry > ov.stoploss && ov.entry < ov.takeprofit {
		_islong = true;
		_quantity = ov.total_liquid * (ov.risk / ((1.0 - (ov.stoploss / ov.entry)) * 100.0));
		_tpslratio = (ov.takeprofit - ov.entry) / (ov.entry - ov.stoploss); //toFixed(2)
	}
	else if ov.stoploss > ov.takeprofit && ov.entry < ov.stoploss && ov.entry > ov.takeprofit {
		_islong = false;
		_quantity = ov.total_liquid * (ov.risk / ((1.0 - (ov.entry / ov.stoploss)) * 100.0));
		_tpslratio = (ov.entry - ov.takeprofit) / (ov.stoploss - ov.entry); //toFixed(2)
	} else {
		bail!("Something bad happened in calculate_order!")
	}

	Ok(OrderCalcExit {
		quantity: _quantity,
		islong: _islong,
		tpslratio: _tpslratio,
	})
}
