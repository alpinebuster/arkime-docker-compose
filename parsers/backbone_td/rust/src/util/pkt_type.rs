#[derive(Debug)]
pub enum PacketDirection {
    Upstream,
    Downstream,
}

#[derive(Debug)]
pub enum FlowLength {
    TOLNet,
    LEXNet,
}
