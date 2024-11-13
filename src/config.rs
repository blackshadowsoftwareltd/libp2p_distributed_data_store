use std::time::Duration;

use anyhow::Result;
use libp2p::{
    kad::{self, store::MemoryStore, Mode},
    mdns, noise,
    swarm::NetworkBehaviour,
    tcp, yamux,
};

use crate::run::{KadMemoryBehav, SwarmBehaviour};

#[derive(NetworkBehaviour)]
pub struct CustomBehaviour {
    pub kademlia: KadMemoryBehav,
    pub mdns: mdns::tokio::Behaviour,
}

pub fn swarm_config() -> Result<SwarmBehaviour> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok(CustomBehaviour {
                kademlia: kad::Behaviour::new(
                    key.public().to_peer_id(),
                    MemoryStore::new(key.public().to_peer_id()),
                ),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
            })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    Ok(swarm)
}
