//! Anonymous order book for bond trading.
//!
//! Orders are posted with ZK proofs of bond ownership (bids) or token balance
//! (asks).  The order book itself is public; individual identity is not.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{Amount, Digest};
use crate::bond::BondId;

/// Opaque order identifier.
pub type OrderId = Digest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    /// Buyer: willing to pay `price` per bond unit.
    Bid,
    /// Seller: willing to accept `price` per bond unit.
    Ask,
}

/// A resting limit order on the book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id:         OrderId,
    pub bond_id:    BondId,
    pub side:       OrderSide,

    /// Price per bond unit in base token micro-units.
    pub price:      Amount,

    /// Number of bond units this order covers.
    pub quantity:   u64,

    /// Remaining unfilled quantity.
    pub remaining:  u64,

    /// ZK commitment to the maker's bond note (for asks) or token note (for bids).
    pub commitment: Digest,

    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// A completed trade between two anonymous counterparties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id:         Digest,
    pub bond_id:    BondId,
    pub price:      Amount,
    pub quantity:   u64,
    pub executed_at: DateTime<Utc>,
    /// Nullifiers spent in this trade (prevents double-spend).
    pub nullifiers: Vec<crate::types::Nullifier>,
}

/// In-memory order book for a single bond series.
///
/// In production this state is replicated across the P2P network and
/// committed to the DarkFi contract layer via ZK state transitions.
#[derive(Debug, Default)]
pub struct OrderBook {
    asks: Vec<Order>,   // sorted ascending by price
    bids: Vec<Order>,   // sorted descending by price
}

impl OrderBook {
    pub fn new() -> Self { Self::default() }

    /// Insert a new order.  Returns the order ID.
    pub fn insert(&mut self, order: Order) -> OrderId {
        let id = order.id;
        match order.side {
            OrderSide::Ask => {
                let pos = self.asks.partition_point(|o| o.price <= order.price);
                self.asks.insert(pos, order);
            }
            OrderSide::Bid => {
                let pos = self.bids.partition_point(|o| o.price >= order.price);
                self.bids.insert(pos, order);
            }
        }
        id
    }

    /// Try to match resting orders, returning matched trades.
    pub fn match_orders(&mut self) -> Vec<Trade> {
        let mut trades = Vec::new();
        while let (Some(ask), Some(bid)) = (self.asks.first(), self.bids.first()) {
            if bid.price < ask.price { break; }

            // Execute at ask price (price-time priority).
            let qty = ask.remaining.min(bid.remaining);
            let trade = Trade {
                id: {
                    let mut h = blake3::Hasher::new();
                    h.update(ask.id.as_bytes());
                    h.update(bid.id.as_bytes());
                    Digest::from(h.finalize())
                },
                bond_id:     ask.bond_id,
                price:       ask.price,
                quantity:    qty,
                executed_at: Utc::now(),
                nullifiers:  vec![ask.commitment, bid.commitment],
            };
            trades.push(trade);

            // Update remaining quantities (simplistic; remove when filled).
            self.asks[0].remaining -= qty;
            self.bids[0].remaining -= qty;
            if self.asks[0].remaining == 0 { self.asks.remove(0); }
            if self.bids[0].remaining == 0 { self.bids.remove(0); }
        }
        trades
    }

    pub fn best_ask(&self) -> Option<&Order> { self.asks.first() }
    pub fn best_bid(&self) -> Option<&Order> { self.bids.first() }

    pub fn spread(&self) -> Option<Amount> {
        let ask = self.best_ask()?.price;
        let bid = self.best_bid()?.price;
        if ask.0 >= bid.0 { Some(Amount(ask.0 - bid.0)) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Digest;

    fn make_order(price: u64, qty: u64, side: OrderSide) -> Order {
        Order {
            id:         Digest::zero(),
            bond_id:    Digest::zero(),
            side,
            price:      Amount(price),
            quantity:   qty,
            remaining:  qty,
            commitment: Digest::zero(),
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    #[test]
    fn orders_match_at_ask_price() {
        let mut book = OrderBook::new();
        book.insert(make_order(100, 10, OrderSide::Ask));
        book.insert(make_order(105, 10, OrderSide::Bid));
        let trades = book.match_orders();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, Amount(100));
        assert_eq!(trades[0].quantity, 10);
    }

    #[test]
    fn no_match_when_bid_below_ask() {
        let mut book = OrderBook::new();
        book.insert(make_order(110, 5, OrderSide::Ask));
        book.insert(make_order(100, 5, OrderSide::Bid));
        let trades = book.match_orders();
        assert!(trades.is_empty());
    }
}
