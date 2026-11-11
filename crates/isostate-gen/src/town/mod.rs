//! What marks a town out: its church, the cross or the hall in its market
//! place, its station once the railway comes, and a lighthouse at a harbour.

mod church;
mod lighthouse;
mod market;
mod station;

pub use church::church;
pub use lighthouse::lighthouse;
pub use market::{market_cross, market_hall};
pub use station::station;
